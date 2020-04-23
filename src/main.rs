extern crate clap;
mod Network;
use clap::App;

use crate::Network::Transmitter;
use std::net::Ipv4Addr;
use std::time::Duration;

fn main(){
    let ttl = 50;
    let mut transmitter = Transmitter::new(ttl);
    let destination = Ipv4Addr::new(1, 1, 1, 1);
    let timeout = Duration::from_secs(1);
    transmitter.ping(destination, timeout);
}