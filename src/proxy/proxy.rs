use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    time::timeout,
};
#[path = "../structs.rs"]
pub mod structs;
use structs::ProxyState;

use crate::structs::{App, AppInputs, LogManager, ModBusLog, ModbusTcp};
use crate::tui::run_tui;

#[path = "../tui.rs"]
mod tui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let write_to_host_stream = TcpStream::connect("127.0.0.1:8081").await?;

    let write_to_host_stream = Arc::new(Mutex::new(write_to_host_stream));

    let proxy_state_main = Arc::new(Mutex::new(ProxyState::init()));

    let (tx, rx) = channel::<ModBusLog>();

    let log_manager = LogManager {
        logs: VecDeque::new(),
        rx,
    };

    let tui_state_clone = Arc::clone(&proxy_state_main);

    let autoscroll = false;

    let app = App {
        log_m: log_manager,
        filtered_funcs_vec: Vec::new(),
        inputs: AppInputs::default(),
        autoscroll,
    };

    tokio::spawn(async move {
        let _ = run_tui(tui_state_clone, app).await;
    });

    loop {
        let (mut stream, peer) = listener.accept().await?;
        let host_clone = Arc::clone(&write_to_host_stream);

        let proxy_state = Arc::clone(&proxy_state_main);

        let tx = tx.clone();
        tokio::spawn(async move {
            loop {
                let buff = match handle(&mut stream).await {
                    Ok(x) => x,
                    Err(_) => {
                        return; // timeout expired or got incorrect data from the proxy
                    }
                };

                let is_normal = check_request(&buff, &proxy_state).await;

                if !is_normal {
                    _ = stream.shutdown().await;
                    return;

                // IF REQUEST NORMAL
                } else {
                    let log = ModBusLog {
                        header: parse_header(&buff),
                        peer,
                        func_code: buff[7],
                    };
                    let _ = tx.send(log); // SEND HEADER LOG TO TUI 

                    let mut guard = host_clone.lock().await;

                    let _ = guard.write_all(&buff).await;

                    let buff_resp = match handle(&mut guard).await {
                        Ok(x) => x,
                        Err(_) => {
                            return; // timeout expired or got incorrect data from the server
                        }
                    };

                    let is_normal = check_response(&buff);
                    if !is_normal {
                        return;
                    }
                    let _ = stream.write_all(&buff_resp).await;
                }
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

async fn check_request(buff: &[u8], proxy_state: &Arc<Mutex<ProxyState>>) -> bool {
    if buff.len() < 7 {
        return false;
    }

    let length: u16 = u16::from_be_bytes([buff[4], buff[5]]);

    let guard = proxy_state.lock().await;
    if guard.is_func_filter && guard.filtered_funcs[buff[7] as usize] {
        return false;
    }
    if (length as usize) != (buff.len() - 6) || (buff[2] != 0 || buff[3] != 0) || buff.len() > 260 {
        return false;
    }

    true
}

fn check_response(buff: &[u8]) -> bool {
    if buff.len() < 7 {
        return false;
    }

    let length: u16 = u16::from_be_bytes([buff[4], buff[5]]);

    if (length as usize) != (buff.len() - 6) || (buff[2] != 0 || buff[3] != 0) || buff.len() > 260 {
        return false;
    }

    true
}

fn parse_header(buff: &[u8]) -> ModbusTcp {
    let transaction_id: u16 = u16::from_be_bytes([buff[0], buff[1]]);

    let protocol_id: u16 = u16::from_be_bytes([buff[2], buff[3]]);

    let length: u16 = u16::from_be_bytes([buff[4], buff[5]]);

    let unit_id: u8 = buff[6];

    ModbusTcp {
        transaction_id,
        protocol_id,
        length,
        unit_id,
    }
}
