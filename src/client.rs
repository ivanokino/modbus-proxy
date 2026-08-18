use std::io::{stdin, Read, Write};
use std::net::TcpStream;

#[repr(C, packed)]
struct ModbusTcp {
    transaction_id: u16,
    protocol_id: u16,
    length: u16,
    unit_id: u8,
}

impl ModbusTcp {
    fn to_be(&mut self) {
        self.transaction_id = self.transaction_id.to_be();
        self.protocol_id = self.protocol_id.to_be();
        self.length = self.length.to_be();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pdu_data: [u8; 5] = [0x03, 0x00, 0x0A, 0x00, 0x05]; // alreagy BE

    let mut buff: [u8; 260] = [0; 260]; // for send

    let mut norm_pack = ModbusTcp {
        transaction_id: 2,
        protocol_id: 0,
        length: 6, //with unit_id too
        unit_id: 1,
    };
    norm_pack.to_be();

    let mut bad_pack = ModbusTcp {
        transaction_id: 2,
        protocol_id: 0,
        length: 10,
        unit_id: 1,
    };
    bad_pack.to_be();

    let total_len = size_of::<ModbusTcp>() + size_of_val(&pdu_data);

    println!("normal packet - 1\nbad packet - 2");

    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let numb: i32 = input.trim().parse()?;

    let buff_ptr = &mut buff as *mut u8;

    let mut stream: TcpStream = TcpStream::connect("127.0.0.1:8080")?;

    if numb == 1 {
        let header_ptr = &norm_pack as *const ModbusTcp as *const u8;

        unsafe {
            std::ptr::copy_nonoverlapping(header_ptr, buff_ptr, std::mem::size_of_val(&norm_pack));
            std::ptr::copy_nonoverlapping(
                pdu_data.as_ptr(),
                buff_ptr.add(size_of::<ModbusTcp>()),
                size_of_val(&pdu_data),
            );
        }

        stream.write_all(&buff[..total_len])?;
    } else {
        let header_ptr = &bad_pack as *const ModbusTcp as *const u8;

        unsafe {
            std::ptr::copy_nonoverlapping(header_ptr, buff_ptr, std::mem::size_of_val(&bad_pack));
            std::ptr::copy_nonoverlapping(
                pdu_data.as_ptr(),
                buff_ptr.add(size_of::<ModbusTcp>()),
                size_of_val(&pdu_data),
            );
        }

        stream.write_all(&buff[..total_len])?;
    }

    let mut recv_buffer = [0; 512];
    let n = stream.read(&mut recv_buffer)?;

    let resp = String::from_utf8_lossy(&recv_buffer[..n]).to_string();

    println!("response : {}", resp);

    Ok(())
}
