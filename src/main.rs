extern crate clap;
extern crate ctrlc;
mod Network;
use clap::{App, Arg};
use crate::Network::Statistics::StatTracker;

use crate::Network::Transmitter;
use std::net::Ipv4Addr;
use std::time::Duration;
use crossbeam::crossbeam_channel::{bounded, Receiver};
use std::str::FromStr;
use std::cmp::{min, max};

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn main(){
    let matches = App::new("Ping")
        .arg(Arg::with_name("IP")
            .help("The IP that is pinged.")
            .required(true)
            .index(1))
        .arg(Arg::with_name("c")
            .short("c")
            .value_name("num_pings")
            .help("# of times to ping")
            .takes_value(true))
        .arg(Arg::with_name("f")
            .short("f")
            .value_name("flood_ping")
            .help("Helps visualize packet loss")
            .takes_value(false))
        .arg(Arg::with_name("timeout")
            .short("W")
            .value_name("timeout_ms")
            .help("# of ms to wait before timeout")
            .takes_value(true))
        .arg(Arg::with_name("TTL")
                 .short("ttl")
                 .value_name("TTL")
                 .help("TTL of packet")
                 .takes_value(true))
        .get_matches();


    let ctrl_c_events = ctrl_channel().unwrap();
    let ttl = match matches.value_of("TTL") {
        Some(ttl)=>{
            match ttl.parse::<u64>() {
                Ok(ttl)=>{
                    ttl as u8
                }
                Err(e)=>{
                    println!("Error parsing TTL!");
                    return;
                }
            }
        }
        None=>{
            50
        }
    };
    let mut statistics = StatTracker::initialize();
    let mut transmitter = Transmitter::new(ttl);
    let timeout = match matches.value_of("timeout") {
        Some(timeout)=>{
            match timeout.parse::<u64>() {
                Ok(millis)=>{
                    Duration::from_millis(max(millis,10))
                }
                Err(e)=>{
                    println!("Error parsing timeout!");
                    return;
                }
            }
        }
        None=>{
            Duration::from_secs(1)
        }
    };
    let limit = match matches.value_of("c") {
        Some(count)=>{
            match count.parse::<i64>() {
                Ok(count)=>{
                    count
                }
                Err(e)=>{
                    println!("Error parsing count!");
                    return;
                }
            }
        }
        None=>{
            //infinite
            -1
        }
    };

    let flood = matches.is_present("f");
    let dest = Ipv4Addr::from_str(matches.value_of("IP").unwrap());
    match dest {
        Ok(destination)=>{
            transmitter.ping(destination, limit, timeout, flood, &mut statistics,ctrl_c_events);
        },
        Err(e)=>{
            println!("Could not parse IP");
            return;
        }
    }

}