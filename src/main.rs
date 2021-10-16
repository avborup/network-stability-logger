use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Network Stability Logger",
    about = "Continuously ping a server and log the roundtrip time."
)]
struct Options {
    #[structopt(
        short = "a",
        long = "address",
        default_value = "https://www.google.com",
        help = "The URL address to ping"
    )]
    address: String,
}

fn main() {
    let opt = Options::from_args();
    println!("{:?}", opt);
}
