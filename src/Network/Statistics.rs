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
    pub rtt_square_sum_ms:u128,
}
impl StatTracker{

    pub fn initialize()->StatTracker{
        let num_packets_sent = 0;
        let num_packets_dropped = 0;
        let rtt_total_time = Duration::new(0,0);
        let rtt_min_time = Duration::from_secs(u64::max_value());
        let rtt_max_time = Duration::new(0,0);
        let rtt_square_sum_ms = 0;
        let initial_start_time = Instant::now();
        StatTracker{
            num_packets_sent,
            initial_start_time,
            num_packets_dropped,
            rtt_total_time,
            rtt_min_time,
            rtt_max_time,
            rtt_square_sum_ms
        }
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
        let millis = rtt.as_millis();
        self.rtt_square_sum_ms += millis*millis;
    }
    //based of off the way linux ping works
    pub fn get_report(&mut self)->String{
        let num_received = self.num_packets_sent-self.num_packets_dropped;
        let packet_loss = self.get_packet_loss();
        let avg_time = (self.rtt_total_time.as_millis() as f64)/num_received as f64;
        let smean = (self.rtt_square_sum_ms as f64) /num_received as f64;
        let mdev = (smean-avg_time*avg_time).abs().sqrt();

        format!("{} packets transmitted, {} received, {:.2}% packet loss, time {:?}\nrtt min/avg/max/mdev = {:?}/{:.3}ms/{:?}/{:.3}ms",
         self.num_packets_sent, num_received, packet_loss, self.initial_start_time.elapsed(), self.rtt_min_time, avg_time,self.rtt_max_time, mdev)
    }
    pub fn get_packet_loss(&mut self) ->f64{
        (self.num_packets_dropped as f64)/(self.num_packets_sent as f64) * 100f64
    }
}