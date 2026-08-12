use std::{
    convert::TryInto,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    let mut write_to_host_stream = TcpStream::connect("127.0.0.1:8081")?;

    for stream in listener.incoming() {
        let mut stream = stream?;

        let buff = handle(&mut stream)?;

        let is_normal = check(&buff);

        if !is_normal {
            stream.write_all("bad package".as_bytes())?;
            stream.shutdown(std::net::Shutdown::Both)?;
            println!("bad package");
        } else {
            stream.write_all("success".as_bytes())?;
            write_to_host_stream.write_all(&buff)?;
        }
    }

    Ok(())
}

fn handle(stream: &mut TcpStream) -> Result<Vec<u8>, std::io::Error> {
    let mut buff = [0; 1024];

    let nofbytes = stream.read(&mut buff)?;

    Ok(buff[..nofbytes].to_vec())
}

fn check(buff: &[u8]) -> bool {
    let bytes: [u8; 2] = buff[4..6].try_into().expect("error in the check()");
    let lenght: u16 = u16::from_be_bytes(bytes);

    if (lenght as usize) != (buff.len() - 6)
        || (buff[2] != 0 && buff[3] != 0)
        || buff.len() < 7
        || buff.len() > 260
    {
        return false;
    }

    println!("lenght: {}", lenght);

    true
}
