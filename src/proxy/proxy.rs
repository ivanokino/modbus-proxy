use std::sync::Arc;
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    time::timeout,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let write_to_host_stream = TcpStream::connect("127.0.0.1:8081").await?;

    let write_to_host_stream = Arc::new(Mutex::new(write_to_host_stream));

    loop {
        let (mut stream, _) = listener.accept().await?;
        let host_clone = Arc::clone(&write_to_host_stream);

        tokio::spawn(async move {
            let buff = match handle(&mut stream).await {
                Ok(x) => x,
                Err(_) => {
                    println!("user: read_exact timeout expired or server got incorrect data");
                    return;
                }
            };

            let is_normal = check(&buff);

            if !is_normal {
                _ = stream.write_all("bad packet".as_bytes()).await;
                _ = stream.shutdown().await;
                println!("bad packet");
            } else {
                let mut guard = host_clone.lock().await;

                println!("writing request");
                let _ = guard.write_all(&buff).await;

                let buff_resp = match handle(&mut guard).await {
                    Ok(x) => x,
                    Err(_) => {
                        println!(
                            "server: read_exact timeout expired or got incorrect data from the server "
                        );
                        return;
                    }
                };

                let is_normal = check(&buff);
                if !is_normal {
                    println!("bad response");

                    return;
                }
                println!("writing resp");
                let _ = stream.write_all(&buff_resp).await;
            }
        });
    }
}

async fn handle(stream: &mut TcpStream) -> Result<Vec<u8>, std::io::Error> {
    let mut header = [0; 7];
    let _ = stream.read_exact(&mut header).await?;

    let body_len = u16::from_be_bytes([header[4], header[5]]) as usize - 1; //-1 cuz of unit id 

    if body_len > 260 || body_len == 0 {
        return Err(std::io::Error::other("incorrect len"));
    }

    let mut body = vec![0u8; body_len];

    timeout(Duration::from_secs(3), stream.read_exact(&mut body)).await??;

    let mut packet = header.to_vec();
    packet.extend_from_slice(&body[..body_len]);

    Ok(packet)
}

fn check(buff: &[u8]) -> bool {
    if buff.len() < 7 {
        return false;
    }

    let length: u16 = u16::from_be_bytes([buff[4], buff[5]]);

    if (length as usize) != (buff.len() - 6) || (buff[2] != 0 || buff[3] != 0) || buff.len() > 260 {
        return false;
    }

    true
}
