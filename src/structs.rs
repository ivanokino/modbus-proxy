#![allow(dead_code)]
use std::{collections::VecDeque, net::SocketAddr, sync::mpsc::Receiver};
pub struct ModbusTcp {
    pub transaction_id: u16,
    pub protocol_id: u16,
    pub length: u16,
    pub unit_id: u8,
}

#[derive(Default)]
pub struct AppInputs {
    pub func_code_input: String,
}

impl ModbusTcp {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(7);
        bytes.extend_from_slice(&self.transaction_id.to_be_bytes());
        bytes.extend_from_slice(&self.protocol_id.to_be_bytes());
        bytes.extend_from_slice(&self.length.to_be_bytes());
        bytes.push(self.unit_id);

        bytes
    }
}

pub struct App {
    pub log_m: LogManager,
    pub inputs: AppInputs,
    pub filtered_funcs_vec: Vec<u8>,
    pub autoscroll: bool,
}

pub struct ModBusLog {
    pub header: ModbusTcp,
    pub peer: SocketAddr,
    pub func_code: u8,
}

pub struct ProxyState {
    pub is_func_filter: bool,
    pub filtered_funcs: [bool; 256],
}

impl ProxyState {
    pub fn init() -> Self {
        ProxyState {
            is_func_filter: false,
            filtered_funcs: [false; 256],
        }
    }
}

pub struct LogManager {
    pub logs: VecDeque<ModBusLog>,
    pub rx: Receiver<ModBusLog>,
}
