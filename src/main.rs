extern crate clap;
mod Network;
use clap::App;

use crate::Network::Transmitter;

fn main(){
    let transmitter = Transmitter{};
    transmitter.ping();
}