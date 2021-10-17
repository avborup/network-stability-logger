use std::collections::VecDeque;
use std::io::{self, Write};
use std::net::IpAddr;
use std::str::FromStr;
use std::thread;
use std::time::{Duration, Instant};
use structopt::StructOpt;
use ui::Ui;

mod ui;

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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Options::from_args();

    let stdout = io::stdout();
    let mut ui = Ui::new(stdout);

    let mut datapoints = (0..=500)
        .into_iter()
        .map(|i| Datapoint {
            timestamp: Instant::now(),
            value: (i as f64 / 10.0 + 1.0).cos(),
            index: i as usize,
        })
        .collect::<VecDeque<_>>();

    for _i in 0..datapoints.len() {
        ui.repaint(datapoints.iter())?;
        ui.flush()?;
        thread::sleep(Duration::from_millis(100));
        datapoints.pop_front();
    }

    Ok(())

    // let ip = IpAddr::from_str(&opt.address)?;
    // let delay = Duration::from_millis(opt.delay);

    // loop {
    //     let now = Instant::now();
    //     let result = ping::ping(ip, Some(Duration::from_secs(1)), None, None, None, None);
    //     let elapsed = now.elapsed().as_millis();

    //     match result {
    //         Ok(_) => println!("{} ms", elapsed),
    //         Err(e) => println!("error: {:?}", e),
    //     }

    //     thread::sleep(delay);
    // }
}
