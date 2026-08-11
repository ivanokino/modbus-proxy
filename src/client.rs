mod proxy;

use std::io::{Read, Write, stdin};
use std::net::TcpStream;



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream: TcpStream = TcpStream::connect("127.0.0.1:8080").expect("error: cant connect to 8080");

    println!("norm paket - 1");
    println!("nenorm paket - 2");

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    let numb: i32 = input.trim().parse().expect("not a num");

    if numb == 1
    {
    stream.write_all(b"\x00\x02\x00\x00\x00\x06\x01\x03\x00\x0A\x00\x05")?;
    }else{
        stream.write_all(b"\\x00\x02\x00\x00\x00\x0A\x01\x03\x00\x0A\x00\x05")?;
    }


    let mut buffer: [u8; 512] = [0; 512];
    let n = stream.read(&mut buffer)?;

    let resp: String = String::from_utf8_lossy(&buffer[..n]).to_string();
    
        println!("response : {}", &resp);

    Ok(())
}