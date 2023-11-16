use std::net::IpAddr;
use std::time::{Duration, Instant};

use futures::future::join_all;

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
    echo_req_packet.set_identifier(0xB000);

    let packet_checksum = checksum(echo_req_packet.packet(), 1);
    echo_req_packet.set_checksum(packet_checksum);

    Ok(echo_req_packet.consume_to_immutable())
}

fn ping(ip_str: String) {
    println!("pinging {}", ip_str);
    let target_ip = IpAddr::V4(ip_str.parse().unwrap());
    let protocol = Layer4(Ipv4(IpNextHeaderProtocols::Icmp));

    let (mut tx, mut rx) =
        transport_channel(64, protocol).expect("failed to make transport channel");

    let mut send_buffer = [0u8; 64];
    let echo_req_packet = match construct_packet(send_buffer.as_mut()) {
        Ok(packet) => packet,
        Err(e) => panic!(
            "failed to construct packet, most likely because the send buffer is too small: {}",
            e
        ),
    };

    let start = Instant::now();
    tx.send_to(echo_req_packet, target_ip)
        .expect("failed to send packet");

    let mut icmp_iter = icmp_packet_iter(&mut rx);

    let packet = match icmp_iter.next_with_timeout(Duration::from_millis(1000)) {
        Ok(response) => response,
        Err(e) => panic!("failed to receive packet: {:?}", e),
    };

    match packet {
        Some((packet, addr)) => match packet.get_icmp_type() {
            IcmpTypes::EchoReply => println!("{} {:?}", addr, start.elapsed()),
            _ => println!("received unexpected packet {:?}", packet),
        },
        None => (),
    }
}

async fn aping(ip_str: String) {
    tokio::task::spawn_blocking(move || ping(ip_str))
        .await
        .expect("The blocking task failed");
}

fn numbers_to_string(n0: u8, n1: u8, n2: u8, n3: u8) -> String {
    // format numbers into ip-like string n0.n1.n2.n3
    format!("{}.{}.{}.{}", n0, n1, n2, n3)
}

#[tokio::main]
async fn main() {
    for i in 0..=255 {
        for j in 0..=255 {
            let mut futs = Vec::new();
            for k in 0..=255 {
                for l in 0..=255 {
                    futs.push(
                        aping(numbers_to_string(i, j, k, l))
                    );
                }
            }
            join_all(futs).await;
        }
    }
}
