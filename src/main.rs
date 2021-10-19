use crossterm::style::Color;
use log::{info, LevelFilter};
use simplelog::WriteLogger;
use std::collections::VecDeque;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::thread;
use std::time::{Duration, Instant};
use structopt::StructOpt;
use ui::Ui;

mod ui;

const BAR_CHART_YELLOW_START: f64 = 25.0;
const BAR_CHART_RED_START: f64 = 75.0;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Network Stability Logger",
    about = "Continuously ping a server and log the roundtrip time."
)]
struct Options {
    #[structopt(
        long = "ip",
        default_value = "8.8.8.8",
        help = "The IP address to ping"
    )]
    address: String,

    #[structopt(
        short = "d",
        long = "delay",
        default_value = "500",
        help = "The delay time between each ping in milliseconds"
    )]
    delay: u64,

    #[structopt(
        parse(from_os_str),
        short = "o",
        long = "output",
        help = "Path to a file where log output should be written"
    )]
    output_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
pub struct Datapoint {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub value: f64,
    pub failed: bool,
}

impl Datapoint {
    pub fn color(&self) -> Color {
        if self.value >= BAR_CHART_RED_START || self.failed {
            Color::DarkRed
        } else if self.value >= BAR_CHART_YELLOW_START {
            Color::Yellow
        } else {
            Color::Green
        }
    }

    pub fn value_str(&self) -> String {
        if !self.failed {
            format!("{} ms", self.value)
        } else {
            String::from("X")
        }
    }

    pub fn time_str(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
    }
}

impl fmt::Display for Datapoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.time_str(), self.value_str())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Options::from_args();

    if let Some(output_path) = opt.output_path {
        WriteLogger::init(
            LevelFilter::Info,
            simplelog::Config::default(),
            fs::File::create(output_path)?,
        )?;
    }

    let stdout = io::stdout();
    let mut ui = Ui::new(stdout);

    const MAX_STORED_DATAPOINTS: usize = 500;
    let mut datapoints = VecDeque::with_capacity(MAX_STORED_DATAPOINTS);

    let ip = IpAddr::from_str(&opt.address)?;
    let delay = Duration::from_millis(opt.delay);

    loop {
        let ping_result = measure_ping(ip);
        let datapoint = Datapoint {
            timestamp: chrono::offset::Local::now(),
            failed: ping_result.is_err(),
            value: ping_result.map(|d| d.as_millis()).unwrap_or_default() as f64,
        };

        info!("{}", datapoint);

        if datapoints.len() >= MAX_STORED_DATAPOINTS {
            datapoints.pop_back();
        }

        datapoints.push_front(datapoint);

        ui.repaint(datapoints.iter())?;
        ui.flush()?;

        thread::sleep(delay);
    }
}

fn measure_ping(ip: IpAddr) -> Result<Duration, Box<dyn std::error::Error>> {
    let now = Instant::now();
    ping::ping(ip, Some(Duration::from_secs(1)), None, None, None, None)?;
    let elapsed = now.elapsed();

    Ok(elapsed)
}
