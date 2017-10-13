// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate clap;
extern crate libmodbus_rs;

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{}
}

use errors::*;
use libmodbus_rs::{Modbus, ModbusClient, ModbusTCP};


fn _test_read_bits(modbus: &Modbus) {
    let address = 1;
    let mut dest = vec![0u8; 100];

    println!("{:?}", modbus.read_bits(address, 1, &mut dest));
}

fn _test_reply_exception(modbus: &Modbus) {
    use libmodbus_rs::Exception;

    let request: Vec<u8> = vec![0x01];
    assert_eq!(modbus.reply_exception(&request, Exception::Acknowledge as u32).unwrap(), 9);
}

fn _test_write_and_read_registers(modbus: &Modbus) {
    let address = 1;
    let request_bytes = vec![1337u16];
    let mut response_bytes = vec![0u16];

    assert_eq!(modbus.write_and_read_registers(address, 1, &request_bytes, address, 1, &mut response_bytes)
                   .unwrap(),
               1);

    println!("reponse_bytes: {:?}", response_bytes);
}

fn _test_raw_request(modbus: &Modbus) {
    use libmodbus_rs::FunctionCode;

    let mut raw_request: Vec<u8> = vec![0xFF, FunctionCode::ReadHoldingRegisters as u8, 0x00, 0x01, 0x0, 0x05];
    let mut response: Vec<u8> = vec![0; Modbus::TCP_MAX_ADU_LENGTH as usize];

    assert_eq!(modbus.send_raw_request(&mut raw_request).unwrap(), 12);
    assert_eq!(modbus.receive_confirmation(&mut response).unwrap(), 19);
    println!(">> response: {:?}", &response);
}


fn run() -> Result<()> {
    let mut modbus = Modbus::new_tcp("127.0.0.1", 1502).chain_err(|| "unable to create modbus TCP context")?;
    modbus.set_debug(true).chain_err(|| "could not set modbus DEBUG mode")?;

    // `accept()` and `listen()` are not working yeat?
    // let mut socket = modbus.tcp_listen(10).chain_err(|| "could not listen")?;
    // modbus.tcp_accept(&mut socket).chain_err(|| "could not accept TCP socket")?;
    modbus.connect().chain_err(|| "could not connect")?;

    // Write and read registers test
    // test_write_and_read_registers(&modbus);
    // Raw request tests
    // test_raw_request(&modbus);
    // test_reply_exception(&modbus);
    // test_read_bits(&modbus);


    Ok(())
}

fn main() {
    if let Err(ref err) = run() {
        println!("Error: {}", err);

        std::process::exit(1)
    }
}
