use std::net::IpAddr;

use pnet::util::checksum;

use pnet::packet::icmp::echo_request::{EchoRequestPacket, MutableEchoRequestPacket};
use pnet::packet::icmp::IcmpTypes;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::Packet;

use pnet::transport::icmp_packet_iter;
use pnet::transport::transport_channel;
use pnet::transport::TransportChannelType::Layer4;
use pnet::transport::TransportProtocol::Ipv4;

fn construct_packet(send_buffer: &mut [u8]) -> Result<EchoRequestPacket, String> {
    let mut echo_req_packet = match MutableEchoRequestPacket::new(send_buffer) {
        Some(packet) => packet,
        None => return Err("failed to construct packet".to_owned()),
    };

    echo_req_packet.set_icmp_type(IcmpTypes::EchoRequest);
    echo_req_packet.set_sequence_number(1);
    echo_req_packet.set_identifier(0x88F8);

    let packet_checksum = checksum(echo_req_packet.packet(), 1);
    echo_req_packet.set_checksum(packet_checksum);

    Ok(echo_req_packet.consume_to_immutable())
}

#[allow(unused_variables)]
fn main() {
    let target_ip = IpAddr::V4("8.8.8.8".parse().unwrap());
    let protocol = Layer4(Ipv4(IpNextHeaderProtocols::Icmp));

    let (mut tx, mut rx) =
        transport_channel(4096, protocol).expect("failed to make transport channel");

    let mut send_buffer = [0u8; 64];
    let echo_req_packet = match construct_packet(send_buffer.as_mut()) {
        Ok(packet) => packet,
        Err(e) => panic!(
            "Failed to construct packet, most likely because of too small of a send buffer: {:?}",
            e
        ),
    };

    tx.send_to(echo_req_packet, target_ip)
        .expect("Failed to send packet");

    let mut icmp_iter = icmp_packet_iter(&mut rx);
    match icmp_iter.next() {
        Ok((packet, addr)) => match packet.get_icmp_type() {
            IcmpTypes::EchoReply => println!("Received Echo Reply"),
            _ => println!("Received unexpected packet"),
        },
        Err(e) => println!("Failed to receive packet: {:?}", e),
    }
}
