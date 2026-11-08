use std::io::{Read, Write, stdin};
use std::net::TcpStream;
use std::time::Duration;

mod packet;
use packet::make_correct_request;
#[path = "../structs.rs"]
mod structs;

use crate::packet::{make_incorrect_request, recv_packet};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("normal packet - 1\nbad packet - 2");

    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let numb: i32 = input.trim().parse()?;

    let mut stream: TcpStream = TcpStream::connect("127.0.0.1:8080")?;

    if numb == 1 {
        let packet = make_correct_request();
        loop {
            stream.write_all(&packet)?;
            recv_packet(&mut stream)?;
            std::thread::sleep(Duration::from_millis(300));
        }
    } else {
        let packet = make_incorrect_request();

        stream.write_all(&packet)?;
        let mut recv_buff = [0; 260];

        let len = stream.read(&mut recv_buff)?;
        println!("{}", String::from_utf8_lossy(&recv_buff[..len]));
        Ok(())
    }
}
