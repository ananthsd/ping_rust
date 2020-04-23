use std::time::{Duration, Instant};
use std::alloc::System;
use std::cmp::{min, max};

pub struct StatTracker{
    pub num_packets_sent:u64,
    pub initial_start_time:Instant,
    pub num_packets_dropped:u64,
    //used for avg
    pub rtt_total_time:Duration,
    pub rtt_min_time:Duration,
    pub rtt_max_time:Duration,
    //used for sdev sqrt(square_mean^2-mean^2)
    pub rtt_square_sum_ms:i128,
}
impl StatTracker{
    pub fn initialize(&mut self){
        self.initial_start_time = Instant::now();
    }
    pub fn register_drop(&mut self){
        self.num_packets_dropped+=1;
        self.num_packets_sent+=1;
    }
    pub fn register_received(&mut self, rtt:Duration){
        self.num_packets_sent+=1;
        self.rtt_total_time+=rtt;
        self.rtt_min_time = min(rtt,self.rtt_min_time);
        self.rtt_max_time = max(rtt,self.rtt_max_time);
        //doing this for precision purposes, but may need to go to millis if this overflows
        let millis = rtt.as_millis()+rtt.subsec_micros() as u128;
        self.rtt_square_sum_micros += millis*millis;
    }
    //based of off the way linux ping works
    pub fn get_report(&mut self)->String{
        let num_received = self.num_packets_sent-self.num_packets_dropped;
        let packet_loss = (self.num_packets_dropped as i64)/(self.num_packets_sent as i64);

        format!("{} packets transmitted, {} received, {}% packet loss, time {:?}\nrtt min/avg/max/mdev = {:?}/{:?}/{:?}/{}",
         self.num_packets_sent, num_received, packet_loss, self.initial_start_time.elapsed(), self.rtt_min_time, ,self.rtt_max_time)
    }
}