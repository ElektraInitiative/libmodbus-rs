use libmodbus::{Modbus, ModbusMapping, ModbusServer, ModbusTCPPI};


fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut modbus = Modbus::new_tcp_pi("::0", "1502")?;
    let mut socket = modbus.tcp_pi_listen(1)?;
    modbus.tcp_pi_accept(&mut socket)?;

    let modbus_mapping = ModbusMapping::new(500, 500, 500, 500)?;
    let mut query = vec![0u8; Modbus::MAX_ADU_LENGTH as usize];

    loop {
        let request_len = modbus.receive(&mut query)?;
        modbus.reply(&query, request_len, &modbus_mapping)?;
    }
}

fn main() {
    if let Err(ref err) = run() {
        println!("Error: {}", err);

        std::process::exit(1)
    }
}
