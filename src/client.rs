use std::io::{Read, Write, stdin};
use std::net::TcpStream;
use std::os::raw;
use std::ptr;
use std::str::from_utf8;
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
            ptr::copy_nonoverlapping(header_ptr, buff_ptr, std::mem::size_of_val(&norm_pack));
            ptr::copy_nonoverlapping(
                pdu_data.as_ptr(),
                buff_ptr.add(size_of::<ModbusTcp>()),
                size_of_val(&pdu_data),
            );
        }

        stream.write_all(&buff[..total_len])?;
    } else {
        let header_ptr = &bad_pack as *const ModbusTcp as *const u8;

        unsafe {
            ptr::copy_nonoverlapping(header_ptr, buff_ptr, std::mem::size_of_val(&bad_pack));
            ptr::copy_nonoverlapping(
                pdu_data.as_ptr(),
                buff_ptr.add(size_of::<ModbusTcp>()),
                size_of_val(&pdu_data),
            );
        }

        stream.write_all(&buff[..total_len])?;
        let mut recv_buff = [0; 260];

        let len = stream.read(&mut recv_buff)?;
        println!("{}", String::from_utf8_lossy(&recv_buff[..len]));
        return Ok(());
    }

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

    let data = &recv_buffer[6..len];

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
    print!("\n");
    Ok(())
}
