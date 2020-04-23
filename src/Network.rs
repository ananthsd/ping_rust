extern crate clap;
use pnet::packet::icmp::{IcmpPacket, IcmpTypes, IcmpCode};
use pnet::packet::icmp::echo_request::MutableEchoRequestPacket;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::transport::{transport_channel, TransportChannelType, icmp_packet_iter, ipv4_packet_iter};
use pnet::transport::TransportProtocol::{Ipv4, Ipv6};
use std::net::{Ipv4Addr, IpAddr};
use pnet::packet::MutablePacket;
use pnet::util::checksum;
use pnet::packet::Packet;
use pnet::packet::PacketSize;
use std::time::Instant;
mod Statistics;


////https://github.com/libpnet/libpnet/blob/master/src/pnettest.rs
const IPV4_HEADER_LEN: usize = 21;
const IPV6_HEADER_LEN: usize = 40;
////http://www.networksorcery.com/enp/protocol/icmp.htm#Code
const TCMP_HEADER_LEN: usize = 8;
const TCMP_DATA_LEN: usize = 32;
const TCMP_LEN: usize = TCMP_HEADER_LEN + TCMP_DATA_LEN;
const TOTAL_LEN: usize = IPV4_HEADER_LEN + TCMP_LEN;
//const TOTAL_LEN:usize = 84;

pub struct Transmitter {

}

impl Transmitter {
    //https://github.com/libpnet/libpnet/blob/master/pnet_packet/src/icmp.rs.in
    pub fn ping(&self) {
        //you need root for this
        let (mut sender, mut receiver) = match transport_channel(4096, TransportChannelType::Layer3(IpNextHeaderProtocols::Icmp)) {
            Ok((s, r)) => { (s, r) }
            Err(e) => { panic!("Could not create sockets:{}", e) }
        };
        // println!("initialized channels");

        let destination = Ipv4Addr::new(1, 1, 1, 1);
        let mut ipv4_buf = [0; 40];
        let mut icmp_buf = [0; 40];
        let ttl = 50;
        let mut sequence_num = 0;
        let packet = self.get_ipv4_packet(&mut ipv4_buf, &mut icmp_buf, ttl, sequence_num, destination);
        // println!("initialized packet :{:?}", packet);
        let mut start = Instant::now();
        let data_sent = match sender.send_to(packet, IpAddr::V4(destination)) {
            Ok(num) => { num }
            Err(e) => { panic!("Packet not sent: {}", e) }
        };
        // println!("sent packet data:{}", data_sent);


        let mut receiver = ipv4_packet_iter(&mut receiver);
        // println!("setup receiver");
        loop {
            // println!("checking receiver");
            let next = receiver.next();
            // println!("waiting");
            match next {
                Ok((packet, addr)) => {
                    // println!("we got a packet: {:?} from {} with size {}", IcmpPacket::new(packet.payload()), addr,packet.packet_size());
                    println!("{} bytes from {}: icmp_seq={} ttl={} time={:?}", packet.packet_size(), addr,sequence_num, ttl,start.elapsed());
                    sequence_num+=1;
                    let packet = self.get_ipv4_packet(&mut ipv4_buf, &mut icmp_buf, ttl, sequence_num, destination);
                    // println!("initialized packet :{:?}", packet);
                    start = Instant::now();
                    let data_sent = match sender.send_to(packet, IpAddr::V4(destination)) {
                        Ok(num) => { num }
                        Err(e) => { panic!("Packet not sent: {}", e) }
                    };
                    // return;
                }
                Err(e) => {
                    panic!("We have an error:{}", e);
                }
            }
        }
    }


    //http://www.networksorcery.com/enp/protocol/icmp.htm
    //https://www.tutorialspoint.com/ipv4/ipv4_packet_structure.htm

    ////https://docs.rs/pnet/0.25.0/pnet/packet/icmp/echo_request/struct.MutableEchoRequestPacket.html
    fn get_payload<'a>(&self, buf: &'a mut [u8], sequence_num: u16) -> MutableEchoRequestPacket<'a> {
        use pnet::packet::icmp::{IcmpPacket, IcmpTypes, IcmpCode};
        use pnet::util::checksum;

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
    fn get_ipv4_packet<'a>(&self, packet_buf: &'a mut [u8], payload_buf: &'a mut [u8], ttl: u8, sequence_num: u16, destination: Ipv4Addr) -> MutableIpv4Packet<'a> {
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
        packet.set_destination(destination);
        packet.set_checksum(pnet::packet::ipv4::checksum(&packet.to_immutable()));
        packet.set_payload(self.get_payload(payload_buf, sequence_num).packet_mut());
        packet
    }

}
