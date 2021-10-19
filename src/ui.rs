// Disclaimer: this module is heavily inspired by the tui-rs crate.
use crate::{BAR_CHART_RED_START, BAR_CHART_YELLOW_START};
use crossterm::{
    cursor::{Hide, MoveTo},
    execute, queue,
    style::{Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::{
    cmp::{self, Ordering},
    io::{self, Write},
};

use crate::Datapoint;

pub struct Ui<W: Write> {
    buffer: W,
    bar_chart_meta_data: BarChartMetaData,
}

struct BarChartMetaData {
    max_bar_value: f64,
    min_bar_value: f64,
    max_seen_value: Option<f64>,
    min_seen_value: Option<f64>,
}

impl<W> Write for Ui<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buffer.flush()
    }
}

impl<W: Write> Ui<W> {
    pub fn new(buffer: W) -> Self {
        return Self {
            buffer,
            bar_chart_meta_data: BarChartMetaData {
                max_bar_value: BAR_CHART_RED_START * 1.25,
                min_bar_value: 0.0,
                max_seen_value: None,
                min_seen_value: None,
            },
        };
    }

    pub fn repaint<'a, I>(&mut self, datapoints: I) -> io::Result<()>
    where
        I: ExactSizeIterator<Item = &'a Datapoint>,
    {
        self.clear()?;

        let areas = self
            .full_terminal_area()?
            .horizontal_split_percentages(&[0.8, 0.2]);

        let v = datapoints.collect::<Vec<_>>();
        Self::calc_bar_chart_metadata(&mut self.bar_chart_meta_data, &v);
        self.draw_bar_chart(v.iter().map(|d| *d), areas[0])?;
        self.draw_data_info(v.iter().map(|d| *d), areas[1])?;

        queue!(
            self.buffer,
            SetForegroundColor(Color::Reset),
            SetBackgroundColor(Color::Reset),
            SetAttribute(Attribute::Reset),
            Hide
        )
    }

    fn clear(&mut self) -> io::Result<()> {
        execute!(self.buffer, Clear(ClearType::All))
    }

    fn full_terminal_area(&self) -> io::Result<Rect> {
        let (width, height) = terminal::size()?;
        Ok(Rect {
            x: 0,
            y: 0,
            width,
            height,
        })
    }

    fn calc_bar_chart_metadata(bcmd: &mut BarChartMetaData, datapoints: &[&Datapoint]) {
        let filtered = datapoints.iter().filter(|d| !d.failed).collect::<Vec<_>>();

        if filtered.is_empty() {
            return;
        }

        let max = filtered
            .iter()
            .map(|d| d.value)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap();
        let min = filtered
            .iter()
            .map(|d| d.value)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap();

        match bcmd.max_seen_value {
            Some(m) if max > m => bcmd.max_seen_value = Some(max),
            None => bcmd.max_seen_value = Some(max),
            _ => {}
        }
        match bcmd.min_seen_value {
            Some(m) if min < m => bcmd.min_seen_value = Some(min),
            None => bcmd.min_seen_value = Some(min),
            _ => {}
        }
    }

    fn draw_bar_chart<'a, I>(&mut self, datapoints: I, area: Rect) -> io::Result<()>
    where
        I: ExactSizeIterator<Item = &'a Datapoint>,
    {
        const BAR_WIDTH: u16 = 1;
        const BAR_GAP: u16 = 1;
        const SYMBOLS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

        let num_items = cmp::min(
            datapoints.len(),
            (area.width / (BAR_WIDTH + BAR_GAP)) as usize,
        );

        let shown_datapoints = datapoints.take(num_items).collect::<Vec<_>>();

        let min = self.bar_chart_meta_data.min_bar_value;
        let max = self.bar_chart_meta_data.max_bar_value;

        queue!(self.buffer, SetForegroundColor(Color::Grey))?;
        for y in [BAR_CHART_YELLOW_START, BAR_CHART_RED_START] {
            let virtual_term_height = 8.0 * area.height as f64;
            let scaled_value = map_range(y, min, max, 0.0, virtual_term_height);
            let term_y = area.bottom() - (scaled_value / 8.0).ceil() as u16;

            queue!(
                self.buffer,
                MoveTo(area.left(), term_y),
                Print("⠁".repeat((area.right() - area.left() - 1) as usize)),
                MoveTo(area.left(), term_y - 1),
                Print(format!("{} ms", y)),
            )?;
        }
        queue!(
            self.buffer,
            MoveTo(area.left(), 0),
            Print("⡀".repeat((area.right() - area.left() - 1) as usize)),
            MoveTo(area.left(), 1),
            Print(format!("{}+ ms", max.round())),
        )?;

        for (i, datapoint) in shown_datapoints.iter().rev().enumerate() {
            queue!(self.buffer, SetForegroundColor(datapoint.color()))?;

            if datapoint.failed {
                let x = BAR_WIDTH / 2 + area.left() + i as u16 * (BAR_WIDTH + BAR_GAP);
                queue!(
                    self.buffer,
                    MoveTo(x, area.bottom()),
                    Print(datapoint.value_str())
                )?;
                continue;
            }

            let virtual_term_height = 8.0 * area.height as f64;
            let scaled_value = map_range(datapoint.value, min, max, 0.0, virtual_term_height);

            for x_offset in 0..BAR_WIDTH {
                let term_x = x_offset + area.left() + i as u16 * (BAR_WIDTH + BAR_GAP);
                let mut draw_bar_block = |y_offset: u16, symbol: char| -> io::Result<()> {
                    let term_y = area.bottom() - y_offset;
                    queue!(self.buffer, MoveTo(term_x, term_y), Print(symbol))
                };

                let num_integer_divs = (scaled_value / 8.0) as u16;
                for j in 0..=num_integer_divs {
                    draw_bar_block(j, SYMBOLS[8])?;
                }

                let remainder = (scaled_value % 8.0) as usize;
                if remainder != 0 || num_integer_divs == 0 {
                    draw_bar_block((scaled_value / 8.0).ceil() as u16, SYMBOLS[remainder])?;
                }
            }
        }

        Ok(())
    }

    fn draw_data_info<'a, I>(&mut self, datapoints: I, area: Rect) -> io::Result<()>
    where
        I: ExactSizeIterator<Item = &'a Datapoint>,
    {
        let num_option_to_str = |n| match n {
            Some(x) => format!("{} ms", x),
            None => String::from("none"),
        };
        let max_val_str = num_option_to_str(self.bar_chart_meta_data.max_seen_value);
        let min_val_str = num_option_to_str(self.bar_chart_meta_data.min_seen_value);

        queue!(
            self.buffer,
            SetForegroundColor(Color::Reset),
            MoveTo(area.left(), area.top()),
            Print(format!("Max: {}", max_val_str)),
            MoveTo(area.left(), area.top() + 1),
            Print(format!("Min: {}", min_val_str)),
        )?;

        const ITEM_HEIGHT: u16 = 2;
        const ITEM_GAP: u16 = 1;
        const INFO_HEIGHT: u16 = 3;

        let num_items = (area.height - INFO_HEIGHT) / (ITEM_HEIGHT + ITEM_GAP);

        for (i, datapoint) in datapoints.take(num_items as usize).enumerate() {
            let y = area.top() + INFO_HEIGHT + i as u16 * (ITEM_HEIGHT + ITEM_GAP);
            queue!(
                self.buffer,
                MoveTo(area.left(), y),
                SetForegroundColor(Color::Reset),
                Print(datapoint.timestamp.format("%Y-%m-%d %H:%M:%S%.3f")),
                MoveTo(area.left(), y + 1),
                SetForegroundColor(datapoint.color()),
                Print(datapoint.value_str()),
            )?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Rect {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl Rect {
    pub fn left(&self) -> u16 {
        self.x
    }

    pub fn right(self) -> u16 {
        self.x.saturating_add(self.width)
    }

    pub fn top(self) -> u16 {
        self.y
    }

    pub fn bottom(&self) -> u16 {
        self.y.saturating_add(self.height)
    }

    pub fn horizontal_split_percentages(&self, pcts: &[f32]) -> Vec<Rect> {
        let mut out = Vec::with_capacity(pcts.len());
        let mut x = self.x;

        for pct in pcts {
            let rect = Rect {
                x,
                y: self.y,
                height: self.height,
                width: (self.width as f32 * pct) as u16,
            };
            x += rect.width;
            out.push(rect);
        }

        out
    }
}

/// Maps a value `n` from the range `a..b` to a new range `c..d`. If the n falls
/// outside of `a..b`, the result is clamped to be within `c..d`.
fn map_range(value: f64, old_min: f64, old_max: f64, new_min: f64, new_max: f64) -> f64 {
    ((value - old_min) / (old_max - old_min) * (new_max - new_min) + new_min)
        .clamp(new_min, new_max)
}
