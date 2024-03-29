extern crate clap;
use pnet::packet::icmp::{IcmpPacket, IcmpTypes, IcmpCode};
use pnet::packet::icmp::echo_request::MutableEchoRequestPacket;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::transport::{transport_channel, TransportChannelType, ipv4_packet_iter, TransportSender};
use std::net::{Ipv4Addr, IpAddr};
use pnet::packet::MutablePacket;
use pnet::util::checksum;
use pnet::packet::Packet;
use pnet::packet::PacketSize;
use std::time::{Instant, Duration};
use crate::Network::Statistics::StatTracker;
use crossbeam::crossbeam_channel::Receiver;

pub(crate) mod Statistics;


////https://github.com/libpnet/libpnet/blob/master/src/pnettest.rs
const IPV4_HEADER_LEN: usize = 21;
// const IPV6_HEADER_LEN: usize = 40;
////http://www.networksorcery.com/enp/protocol/icmp.htm#Code
const TCMP_HEADER_LEN: usize = 8;
const TCMP_DATA_LEN: usize = 32;
const TCMP_LEN: usize = TCMP_HEADER_LEN + TCMP_DATA_LEN;
const TOTAL_LEN: usize = IPV4_HEADER_LEN + TCMP_LEN;
//const TOTAL_LEN:usize = 84;

pub struct Transmitter {
    pub ttl:u8,
    sequence_num:u16
}

impl Transmitter {
    pub fn new(ttl:u8)->Transmitter{
        Transmitter{
            ttl,
            sequence_num:0
        }
    }

    //https://github.com/libpnet/libpnet/blob/master/pnet_packet/src/icmp.rs.in
    pub fn ping(&mut self, mut destination:Ipv4Addr, limit:i64, timeout:Duration, flood:bool, statistics:&mut StatTracker, ctrl_reciever:Receiver<()>) {
        if limit == 0 {
            return;
        }
        //you need root for this
        let (mut sender, mut receiver) = match transport_channel(4096, TransportChannelType::Layer3(IpNextHeaderProtocols::Icmp)) {
            Ok((s, r)) => { (s, r) }
            Err(e) => { panic!("Could not create sockets:{}", e) }
        };
        // println!("initialized channels");
        let mut num_sent = 0;
        if flood {
            print!(".");
        }
        self.send_ipv4_packet(&mut sender, &mut destination);
        num_sent+=1;
        let mut start = Instant::now();
        // println!("sent packet data:{}", data_sent);
        let mut receiver = ipv4_packet_iter(&mut receiver);
        let mut send_packet = false;
        // println!("setup receiver");
        loop {
            //check for exit condition
            match ctrl_reciever.try_recv(){
                Ok(())=>{
                    //we found ctrl-c
                    println!("\n{}",statistics.get_report());
                    return;
                },
                Err(_)=>{}
            }
            let next = receiver.next_with_timeout(timeout);
            // println!("waiting");

            match next {
                Ok(option) => {
                    match option {
                        Some((packet, addr)) => {
                            // println!("we got a packet: {:?} from {} with size {}", IcmpPacket::new(packet.payload()).get_icmp_type(), addr,packet.packet_size());
                            let rtt = start.elapsed();
                            match IcmpPacket::new(packet.payload()) {
                                Some(icmp)=>{
                                    // println!("{:?}",icmp.get_icmp_type());
                                    if icmp.get_icmp_type() == IcmpTypes::TimeExceeded{
                                        statistics.register_ttl_exceeded();
                                        if flood {
                                            print!("");
                                        }
                                        else{
                                            println!("icmp_seq={} TTL of {} exceeded",self.sequence_num, self.ttl);
                                        }
                                    }
                                    else{
                                        statistics.register_received(rtt);
                                        if flood{
                                            print!("{}",(8u8 as char));
                                        }
                                        else {
                                            println!("{} bytes from {}: icmp_seq={} ttl={} loss={:.2}% time={:?}", packet.packet_size(), addr,self.sequence_num, self.ttl,statistics.get_packet_loss(),rtt);
                                        }
                                    }
                                },
                                None=>{}
                            }
                            self.sequence_num+=1;
                            if limit > 0 && num_sent >= limit {
                                println!("\n{}",statistics.get_report());
                                return;
                            }
                            if flood {
                                print!(".");
                            }
                            send_packet = true;

                            // return;
                        }
                        None=>{
                            //timeout
                            //we have dropped the packet
                            if flood {
                                print!("");
                            }
                            else{
                                println!("Packet dropped");
                            }
                            statistics.register_drop();
                            if limit > 0 && num_sent >= limit {
                                println!("\n{}", statistics.get_report());
                                return;
                            }
                            if flood {
                                print!(".");
                            }
                            // println!("initialized packet :{:?}", packet);
                            send_packet = true;
                        }
                    }
                }
                Err(e) => {
                    println!("Error");
                    panic!("We have an error:{}", e);
                }
            }
            if send_packet {
                self.send_ipv4_packet(&mut sender, &mut destination);
                num_sent+=1;
                start = Instant::now();
                send_packet = false;
            }
        }
        // println!("Done loop");
    }


    fn send_ipv4_packet(&mut self,sender:&mut TransportSender, destination: &mut Ipv4Addr)->usize{
        // println!("send packet");
        let mut ipv4_buf = [0; 40];
        let mut icmp_buf = [0; 40];
        // println!("initialized packet :{:?}", packet);
        let packet = self.get_ipv4_packet(&mut ipv4_buf, &mut icmp_buf, self.ttl, self.sequence_num, destination);
        match sender.send_to(packet, IpAddr::V4(destination.clone())) {
            Ok(num) => { num }
            Err(e) => { panic!("Packet not sent: {}", e) }
        }
    }

    //http://www.networksorcery.com/enp/protocol/icmp.htm
    //https://www.tutorialspoint.com/ipv4/ipv4_packet_structure.htm

    ////https://docs.rs/pnet/0.25.0/pnet/packet/icmp/echo_request/struct.MutableEchoRequestPacket.html
    fn get_payload<'a>(&self, buf: &'a mut [u8], sequence_num: u16) -> MutableEchoRequestPacket<'a> {

        let mut payload = match MutableEchoRequestPacket::new(buf) {
            Some(p) => { p }
            None => { panic!("Could not construct payload: {}") }
        };
//8
        payload.set_icmp_type(IcmpTypes::EchoRequest);
//    payload.set_icmp_code(IcmpCodes::NoCode);
        payload.set_icmp_code(IcmpCode::new(0));
        payload.set_sequence_number(sequence_num);
//avoid double mutable borrow
        let checksum = checksum(&payload.packet_mut(), 2);
        payload.set_checksum(checksum);
        // println!("payload: {:?}", payload);
        payload
    }

    ////https://docs.rs/pnet/0.25.0/pnet/packet/ipv4/struct.MutableIpv4Packet.html
    fn get_ipv4_packet<'a>(&self, packet_buf: &'a mut [u8], payload_buf: &'a mut [u8], ttl: u8, sequence_num: u16, destination: &mut Ipv4Addr) -> MutableIpv4Packet<'a> {
        let mut packet = match MutableIpv4Packet::new(packet_buf) {
            Some(p) => { p }
            None => { panic!("Could not create ipv4 packet") }
        };
//ipv4
        packet.set_version(4);
        packet.set_header_length(IPV4_HEADER_LEN as u8);
        packet.set_total_length(TOTAL_LEN as u16);
        packet.set_ttl(ttl);
        packet.set_next_level_protocol(IpNextHeaderProtocols::Icmp);
//    packet.set_source(source);
        packet.set_destination(destination.clone());
        packet.set_checksum(pnet::packet::ipv4::checksum(&packet.to_immutable()));
        packet.set_payload(self.get_payload(payload_buf, sequence_num).packet_mut());
        packet
    }

}
