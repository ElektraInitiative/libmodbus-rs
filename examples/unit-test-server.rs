#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(dead_code)]

mod unit_test_config;

use libmodbus::{Modbus, ModbusMapping, ModbusClient, ModbusServer, ModbusTCP, ModbusTCPPI, ModbusRTU};
use libmodbus::Exception;
use libmodbus::prelude::*;
use std::env;
use std::thread::sleep;
use std::time::Duration;
use unit_test_config::*;


fn run() -> Result<(), Box<dyn std::error::Error>> {
    let backend;

    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        match args[1].to_lowercase().as_ref() {
            "tcp" => backend = Backend::TCP,
            "tcppi" => backend = Backend::TCPPI,
            "rtu" => backend = Backend::RTU,
            _ => {
                println!("Usage:\n  {} [tcp|tcppi|rtu] - Modbus server for unit testing\n\n", args[0]);
                std::process::exit(-1);
            },
        }
    } else {
        /* By default */
        backend = Backend::TCP;
    }

    // Setup modbus context
    let (mut modbus, mut query) = match backend {
        Backend::TCP => {
            (Modbus::new_tcp("127.0.0.1", 1502).unwrap(),
            vec![0u8; Modbus::TCP_MAX_ADU_LENGTH as usize])
        },
        Backend::TCPPI => {
            (Modbus::new_tcp_pi("::0", "1502").unwrap(),
            vec![0u8; Modbus::TCP_MAX_ADU_LENGTH as usize])
        },
        Backend::RTU => {
            (Modbus::new_rtu("/dev/ttyUSB0", 115200, 'N', 8, 1).unwrap(),
            vec![0u8; Modbus::RTU_MAX_ADU_LENGTH as usize])
        },
    };

    if backend == Backend::RTU {
        modbus.set_slave(SERVER_ID)?;
    }

    let header_length = modbus.get_header_length() as usize;

    modbus.set_debug(true).expect("could not set modbus DEBUG mode");

    let modbus_mapping =
        ModbusMapping::new_start_address(UT_BITS_ADDRESS,
                                         UT_BITS_NB,
                                         UT_INPUT_BITS_ADDRESS,
                                         UT_INPUT_BITS_NB,
                                         UT_REGISTERS_ADDRESS,
                                         UT_REGISTERS_NB,
                                         UT_INPUT_REGISTERS_ADDRESS,
                                         UT_INPUT_REGISTERS_NB)?;

    /* Examples from PI_MODBUS_300.pdf.
       Only the read-only input values are assigned. */

    /* Initialize input values that's can be only done server side. */
    set_bits_from_bytes(modbus_mapping.get_input_bits_mut(), 0, UT_INPUT_BITS_NB,
                        &UT_INPUT_BITS_TAB);

    /* Initialize values of INPUT REGISTERS */
    for i in 0..UT_INPUT_REGISTERS_NB {
        modbus_mapping.get_input_registers_mut()[i as usize] = UT_INPUT_REGISTERS_TAB[i as usize];
    }

    match backend {
        Backend::TCP => {
            let mut socket = modbus.tcp_listen(1)?;
            modbus.tcp_accept(&mut socket)?;
        },
        Backend::TCPPI => {
            let mut socket = modbus.tcp_pi_listen(1)?;
            modbus.tcp_pi_accept(&mut socket)?;
        },
        Backend::RTU => {
            modbus.connect()?;
        },
    }

    loop {
        let mut rc: std::result::Result<i32, Error>;
        loop {
            rc = modbus.receive(&mut query);
            /* Filtered queries return 0 */
            match rc {
                Ok(0) => {}
                _ => break,
            }
        }

        /* The connection is not closed on errors which require on reply such as
            bad CRC in RTU. */
        if rc.is_err() && rc.as_ref().unwrap_err().to_string() != "Invalid CRC" {
            /* Quit */
            break;
        }

        /* Special server behavior to test client */
        if query[header_length] == 0x03 {
            /* Read holding registers */

            if query[header_length + 3] as u16 == UT_REGISTERS_NB_SPECIAL {
                println!("Set an incorrect number of values");
                query[header_length + 3] = (UT_REGISTERS_NB_SPECIAL - 1) as u8;
            } else if query[header_length + 1] as u16 == UT_REGISTERS_ADDRESS_SPECIAL {
                println!("Reply to this special register address by an exception");
                modbus.reply_exception(&query, Exception::SlaveDeviceBusy).unwrap();
                continue;
            } else if query[header_length + 1] as u16 == UT_REGISTERS_ADDRESS_INVALID_TID_OR_SLAVE {
                const RAW_REQ_LENGTH: usize = 5;
                let mut raw_req = vec![
                    if backend == Backend::RTU { INVALID_SERVER_ID } else { 0xFF },
                    0x03,
                    0x02, 0x00, 0x00
                ];

                println!("Reply with an invalid TID or slave");
                modbus.send_raw_request(&mut raw_req, RAW_REQ_LENGTH).unwrap();
                continue;
            } else if query[header_length + 1] as u16 == UT_REGISTERS_ADDRESS_SLEEP_500_MS {
                println!("Sleep 0.5 s before replying");
                sleep(Duration::from_millis(500));
            } else if query[header_length + 1] as u16 == UT_REGISTERS_ADDRESS_BYTE_SLEEP_5_MS {
                /* Test low level only available in TCP mode */
                /* Catch the reply and send reply byte a byte */
                let mut req = vec![0x00, 0x1C, 0x00, 0x00, 0x00, 0x05, 0xFF, 0x03, 0x02, 0x00, 0x00];
                let req_length = 11;
                let w_s = modbus.get_socket();
                if w_s.is_err() {
                    println!("Unable to get a valid socket in special test");
                    continue;
                }

                /* Copy TID */
                req[1] = query[1];
                for i in 0..req_length {
                    println!("({:2X})", req[i]);
                    sleep(Duration::from_millis(5));
                    // let rc = send(w_s, (const char*)(req + i), 1, MSG_NOSIGNAL);
                    // if rc.is_err() {
                    //     break;
                    // }
                }
                continue;
            }
        }

        let rc = modbus.reply(&query, rc.unwrap(), &modbus_mapping);
        if rc.is_err() { break; }

    }

    // print!("Quit the loop: %s\n", modbus_strerror(errno));
    println!("Quit the loop: ");


    Ok(())
}


fn main() {
    if let Err(ref err) = run() {
        println!("Error: {}", err);

        std::process::exit(1)
    }
}
