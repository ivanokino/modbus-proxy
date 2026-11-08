use std::{io::Read, net::TcpStream};

use crate::structs::ModbusTcp;

pub fn make_correct_request() -> Vec<u8> {
    let header = ModbusTcp {
        transaction_id: 2,
        protocol_id: 0,
        length: 6, //with unit_id too
        unit_id: 1,
    };

    let pdu_data: [u8; 5] = [0x03, 0x00, 0x0A, 0x00, 0x05]; // alreagy BE

    let mut packet = header.to_bytes();

    packet.extend_from_slice(&pdu_data);
    packet
}
pub fn make_incorrect_request() -> Vec<u8> {
    let header = ModbusTcp {
        transaction_id: 2,
        protocol_id: 0,
        length: 120, //with unit_id too
        unit_id: 1,
    };

    let pdu_data: [u8; 11] = [
        0x03, 0x00, 0x0A, 0x00, 0x05, 0xff, 0xfa, 0xaa, 0xff, 0xff, 0xff,
    ]; // alreagy BE

    let mut packet = header.to_bytes();

    packet.extend_from_slice(&pdu_data);
    packet
}

pub fn recv_packet(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut recv_buffer = [0; 512];
    let _ = stream.read(&mut recv_buffer)?;

    let trans_id: [u8; 2] = recv_buffer[0..2].try_into()?;
    let proto_id = recv_buffer[2..4].try_into()?;
    let length = recv_buffer[4..6].try_into()?;

    let resp_s = ModbusTcp {
        transaction_id: u16::from_be_bytes(trans_id),
        protocol_id: u16::from_be_bytes(proto_id),
        length: u16::from_be_bytes(length),
        unit_id: recv_buffer[6],
    };
    let len = resp_s.length as usize;
    println!("{}", len);
    if len == 0 {
        return Err(Box::from("len == 0"));
    }
    let data = &recv_buffer[7..6 + len];

    print!(
        "RESPONSE:\n
transaction_id: {}\n
protocol_id: {}\n
length: {}\n
unit_id: {}\n
data: ",
        { resp_s.transaction_id },
        { resp_s.protocol_id },
        { resp_s.length },
        { resp_s.unit_id }
    );

    for i in data {
        print!("0x{:02x} ", i);
    }
    Ok(())
}
