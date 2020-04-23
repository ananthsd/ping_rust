extern crate clap;
extern crate ctrlc;
mod Network;
use clap::App;
use crate::Network::Statistics::StatTracker;

use crate::Network::Transmitter;
use std::net::Ipv4Addr;
use std::time::Duration;
use crossbeam::crossbeam_channel::{bounded, Receiver};

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn main(){
    let ctrl_c_events = ctrl_channel().unwrap();
    let ttl = 50;
    let mut statistics = StatTracker::initialize();
    let mut transmitter = Transmitter::new(ttl);
    let destination = Ipv4Addr::new(1, 1, 1, 1);
    let timeout = Duration::from_secs(1);
    transmitter.ping(destination, timeout, &mut statistics,ctrl_c_events);
}