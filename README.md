# Network stability logger
A simple CLI tool that continuously pings an IP address and shows the roundtrip time.

## Demo
![](https://i.imgur.com/faPkxua.gif)

Note: since I made this screen recording, the format has changed slightly:

![image](https://user-images.githubusercontent.com/16561050/137892547-b7b02ecb-30dd-4b66-b76f-0ae62c0d4526.png)


*Hopefully your internet connection is not as poor as the one in this demo!*

## Installation
```
cargo install --git https://github.com/avborup/network-stability-logger
```

## Usage
There are no mandatory arguments. To see all options (IP address to ping, log output file, delay between each ping), run:
```
network-stability-logger --help
```

If you're on Linux or macOS and the program purely outputs red Xs, you may have to run it with `sudo` - see https://github.com/avborup/network-stability-logger/issues/1.
