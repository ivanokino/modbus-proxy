use std::net::TcpStream;
use std::time::Duration;
use std::{
    env,
    io::{Read, Write, stdin},
};

mod packet;
use packet::make_correct_request;
#[path = "../structs.rs"]
mod structs;

use crate::packet::{make_incorrect_request, recv_packet};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect();
    let (interval, port) = get_args(args);

    println!("normal packet - 1\nbad packet - 2");

    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let numb: i32 = input.trim().parse()?;

    let mut stream: TcpStream = TcpStream::connect(format!("127.0.0.1:{}", port))?;

    if numb == 1 {
        let packet = make_correct_request();
        loop {
            stream.write_all(&packet)?;
            recv_packet(&mut stream)?;
            std::thread::sleep(Duration::from_millis(interval));
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

fn get_args(args: Vec<String>) -> (u64, u16) {
    let mut interval = 300;
    let mut port = 8080;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-i" | "--interval" => {
                if let Some(val) = iter.next() {
                    interval = val.parse().unwrap_or(300);
                };
            }

            "-p" | "--port" => {
                if let Some(val) = iter.next() {
                    port = val.parse().unwrap_or(3);
                };
            }

            _ => (),
        }
    }

    (interval, port)
}
