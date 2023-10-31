use std::net::IpAddr;

// use pnet::util::checksum;

use pnet::packet::icmp::echo_request::MutableEchoRequestPacket;
use pnet::packet::icmp::IcmpTypes;
use pnet::packet::ip::IpNextHeaderProtocols;

use pnet::transport::icmp_packet_iter;
use pnet::transport::transport_channel;
use pnet::transport::TransportChannelType::Layer4;
use pnet::transport::TransportProtocol::Ipv4;

#[allow(unused_variables)]
fn main() {
    let target_ip = IpAddr::V4("8.8.8.8".parse().unwrap());
    let protocol = Layer4(Ipv4(IpNextHeaderProtocols::Icmp));

    // Create a new transport channel, dealing with layer 4 packets
    // It has a receive buffer of 4096 bytes.
    let (mut tx, mut rx) =
        transport_channel(4096, protocol).expect("failed to make transport channel");

    let mut send_buffer = [0u8; 64];
    let mut echo_req_packet = MutableEchoRequestPacket::new(&mut send_buffer).unwrap();
    echo_req_packet.set_icmp_type(IcmpTypes::EchoRequest);
    echo_req_packet.set_sequence_number(1);
    echo_req_packet.set_identifier(0xCAFE);
    echo_req_packet.set_checksum(0x2d00); // from wireshark lol

    tx.send_to(echo_req_packet, target_ip)
        .expect("Failed to send packet");

    println!("Waiting for ICMP Echo Reply...");
    // listen for reply
    let mut icmp_iter = icmp_packet_iter(&mut rx);
    loop {
        match icmp_iter.next() {
            Ok((packet, addr)) => match packet.get_icmp_type() {
                IcmpTypes::EchoReply => {
                    println!("Received Echo Reply");
                }
                _ => {
                    println!("Received unexpected packet");
                }
            },
            Err(e) => {
                println!("Failed to receive packet: {:?}", e);
            }
        }
    }
}
