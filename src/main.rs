use std::collections::VecDeque;
use std::io::{self, Write};
use std::net::IpAddr;
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
}

#[derive(Debug, Clone, Copy)]
pub struct Datapoint {
    pub timestamp: Instant,
    pub value: f64,
    pub index: usize,
    pub failed: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Options::from_args();

    let stdout = io::stdout();
    let mut ui = Ui::new(stdout);

    const MAX_STORED_DATAPOINTS: usize = 500;
    let mut datapoints = VecDeque::with_capacity(MAX_STORED_DATAPOINTS);

    let ip = IpAddr::from_str(&opt.address)?;
    let delay = Duration::from_millis(opt.delay);

    let mut index = 0;
    loop {
        let ping_result = measure_ping(ip);
        let datapoint = Datapoint {
            index,
            timestamp: Instant::now(),
            failed: ping_result.is_err(),
            value: ping_result.map(|d| d.as_millis()).unwrap_or_default() as f64,
        };

        if datapoints.len() >= MAX_STORED_DATAPOINTS {
            datapoints.pop_back();
        }

        datapoints.push_front(datapoint);

        ui.repaint(datapoints.iter())?;
        ui.flush()?;

        index += 1;
        thread::sleep(delay);
    }
}

fn measure_ping(ip: IpAddr) -> Result<Duration, Box<dyn std::error::Error>> {
    let now = Instant::now();
    ping::ping(ip, Some(Duration::from_secs(1)), None, None, None, None)?;
    let elapsed = now.elapsed();

    Ok(elapsed)
}
