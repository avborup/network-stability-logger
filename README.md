# Network stability logger
A simple CLI tool that continuously pings an IP address and shows the roundtrip time.

## Demo
![](https://i.imgur.com/faPkxua.gif)

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
