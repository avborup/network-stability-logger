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
    max_bar_value: f64,
    min_bar_value: f64,
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
            max_bar_value: BAR_CHART_RED_START,
            min_bar_value: 0.0,
        };
    }

    pub fn repaint<'a, I>(&mut self, datapoints: I) -> io::Result<()>
    where
        I: ExactSizeIterator<Item = &'a Datapoint>,
    {
        self.clear()?;

        let area = self.full_terminal_area()?;
        self.draw_bar_chart(datapoints, area)?;

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

    fn draw_bar_chart<'a, I>(&mut self, datapoints: I, area: Rect) -> io::Result<()>
    where
        I: ExactSizeIterator<Item = &'a Datapoint>,
    {
        const BAR_WIDTH: u16 = 1;
        const BAR_GAP: u16 = 1;
        const SYMBOLS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

        let num_items = cmp::min(
            datapoints.len(),
            (area.width / (BAR_WIDTH + BAR_GAP) - 1) as usize,
        );

        if num_items == 0 {
            return Ok(());
        }

        let shown_datapoints = datapoints.take(num_items).collect::<Vec<_>>();
        let max = shown_datapoints
            .iter()
            .map(|d| d.value)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap();
        let min = shown_datapoints
            .iter()
            .map(|d| d.value)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap();

        if max > self.max_bar_value {
            self.max_bar_value = max;
        }
        if min < self.min_bar_value {
            self.min_bar_value = min;
        }

        let (min, max) = (self.min_bar_value, self.max_bar_value);

        for (i, datapoint) in shown_datapoints.iter().rev().enumerate() {
            let virtual_term_height = 8.0 * area.height as f64;
            let scaled_value = map_range(datapoint.value, min, max, 0.0, virtual_term_height);

            let color = if datapoint.value < BAR_CHART_YELLOW_START {
                Color::Green
            } else if datapoint.value < BAR_CHART_RED_START {
                Color::Yellow
            } else {
                Color::DarkRed
            };

            queue!(self.buffer, SetForegroundColor(color))?;

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
}

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

    pub fn bottom(&self) -> u16 {
        self.y.saturating_add(self.height)
    }
}

fn map_range(value: f64, old_min: f64, old_max: f64, new_min: f64, new_max: f64) -> f64 {
    (value - old_min) / (old_max - old_min) * (new_max - new_min) + new_min
}
