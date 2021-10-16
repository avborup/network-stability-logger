use std::net::IpAddr;
use std::str::FromStr;
use std::thread;
use std::time::{Duration, Instant};
use structopt::StructOpt;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Options::from_args();

    let ip = IpAddr::from_str(&opt.address)?;
    let delay = Duration::from_millis(opt.delay);

    loop {
        let now = Instant::now();
        let result = ping::ping(ip, Some(Duration::from_secs(1)), None, None, None, None);
        let elapsed = now.elapsed().as_millis();

        match result {
            Ok(_) => println!("{} ms", elapsed),
            Err(e) => println!("error: {:?}", e),
        }

        thread::sleep(delay);
    }
}
