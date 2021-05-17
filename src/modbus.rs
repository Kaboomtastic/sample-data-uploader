#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FunctionTypes {
    ReadCoilStatus = 0x01,
    ReadInputStatus = 0x02,
    ReadHoldingRegisters = 0x03,
    ReadInputRegisters = 0x04,
    WriteSingleCoil = 0x05,
    WriteSingleRegister = 0x06,
    WriteMultipleCoils = 0x15,
    WriteMultipleRegisters = 0x16,
    None,
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize, PartialEq)]
pub enum DataTypes {
    Coil,
    Input,
    Register,
    None,
}
impl DataTypes {
    pub fn new(v: u8) -> Self {
        match v {
            0 => DataTypes::Coil,
            1 => DataTypes::Input,
            2 => DataTypes::Register,
            _ => DataTypes::None,
        }
    }
}

impl FunctionTypes {
    fn new_function_type(val: u8) -> Self {
        match val {
            0x01 => FunctionTypes::ReadCoilStatus,
            0x02 => FunctionTypes::ReadInputStatus,
            0x03 => FunctionTypes::ReadHoldingRegisters,
            0x04 => FunctionTypes::ReadInputRegisters,
            0x05 => FunctionTypes::WriteSingleCoil,
            0x06 => FunctionTypes::WriteSingleRegister,
            0x15 => FunctionTypes::WriteMultipleCoils,
            0x16 => FunctionTypes::WriteMultipleRegisters,
            _ => FunctionTypes::None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ModbusMessage {
    pub address: u8,
    pub function: FunctionTypes,
    pub data: Vec<u8>,
    pub crc: [u8; 2],
    pub data_type: DataTypes,
    pub start_address: u16,
    pub num_data_points: u16,
}

fn calculate_crc(message: ModbusMessage) -> [u8; 2] {
    let mut ret: [u8; 2] = [0; 2];
    let mut data: Vec<u8> = Vec::new();

    data.push(message.address);
    data.push(message.function.clone() as u8);
    data.push((message.start_address >> 8) as u8);
    data.push((message.start_address & 0x0FF) as u8);

    match message.function {
        FunctionTypes::WriteSingleCoil
        | FunctionTypes::ReadCoilStatus
        | FunctionTypes::WriteSingleRegister
        | FunctionTypes::None => {}
        FunctionTypes::ReadHoldingRegisters
        | FunctionTypes::ReadInputRegisters
        | FunctionTypes::ReadInputStatus
        | FunctionTypes::WriteMultipleCoils
        | FunctionTypes::WriteMultipleRegisters => {
            data.push((message.num_data_points >> 8) as u8);
            data.push((message.num_data_points & 0x0FF) as u8);
        }
    }

    match message.function {
        FunctionTypes::WriteSingleCoil
        | FunctionTypes::WriteSingleRegister
        | FunctionTypes::WriteMultipleCoils
        | FunctionTypes::WriteMultipleRegisters => data.append(&mut message.data.clone()),
        _ => {}
    }

    let crc_int = crc_helper(data);
    ret[0] = ((crc_int as u16) & 0x0FF) as u8;
    ret[1] = (((crc_int as u16) >> 8) & 0x0FF) as u8;

    ret
}

fn crc_helper(data: Vec<u8>) -> u16 {
    let mut len = data.len();
    let crc_table = [
        0x0000, 0xC0C1, 0xC181, 0x0140, 0xC301, 0x03C0, 0x0280, 0xC241, 0xC601, 0x06C0, 0x0780,
        0xC741, 0x0500, 0xC5C1, 0xC481, 0x0440, 0xCC01, 0x0CC0, 0x0D80, 0xCD41, 0x0F00, 0xCFC1,
        0xCE81, 0x0E40, 0x0A00, 0xCAC1, 0xCB81, 0x0B40, 0xC901, 0x09C0, 0x0880, 0xC841, 0xD801,
        0x18C0, 0x1980, 0xD941, 0x1B00, 0xDBC1, 0xDA81, 0x1A40, 0x1E00, 0xDEC1, 0xDF81, 0x1F40,
        0xDD01, 0x1DC0, 0x1C80, 0xDC41, 0x1400, 0xD4C1, 0xD581, 0x1540, 0xD701, 0x17C0, 0x1680,
        0xD641, 0xD201, 0x12C0, 0x1380, 0xD341, 0x1100, 0xD1C1, 0xD081, 0x1040, 0xF001, 0x30C0,
        0x3180, 0xF141, 0x3300, 0xF3C1, 0xF281, 0x3240, 0x3600, 0xF6C1, 0xF781, 0x3740, 0xF501,
        0x35C0, 0x3480, 0xF441, 0x3C00, 0xFCC1, 0xFD81, 0x3D40, 0xFF01, 0x3FC0, 0x3E80, 0xFE41,
        0xFA01, 0x3AC0, 0x3B80, 0xFB41, 0x3900, 0xF9C1, 0xF881, 0x3840, 0x2800, 0xE8C1, 0xE981,
        0x2940, 0xEB01, 0x2BC0, 0x2A80, 0xEA41, 0xEE01, 0x2EC0, 0x2F80, 0xEF41, 0x2D00, 0xEDC1,
        0xEC81, 0x2C40, 0xE401, 0x24C0, 0x2580, 0xE541, 0x2700, 0xE7C1, 0xE681, 0x2640, 0x2200,
        0xE2C1, 0xE381, 0x2340, 0xE101, 0x21C0, 0x2080, 0xE041, 0xA001, 0x60C0, 0x6180, 0xA141,
        0x6300, 0xA3C1, 0xA281, 0x6240, 0x6600, 0xA6C1, 0xA781, 0x6740, 0xA501, 0x65C0, 0x6480,
        0xA441, 0x6C00, 0xACC1, 0xAD81, 0x6D40, 0xAF01, 0x6FC0, 0x6E80, 0xAE41, 0xAA01, 0x6AC0,
        0x6B80, 0xAB41, 0x6900, 0xA9C1, 0xA881, 0x6840, 0x7800, 0xB8C1, 0xB981, 0x7940, 0xBB01,
        0x7BC0, 0x7A80, 0xBA41, 0xBE01, 0x7EC0, 0x7F80, 0xBF41, 0x7D00, 0xBDC1, 0xBC81, 0x7C40,
        0xB401, 0x74C0, 0x7580, 0xB541, 0x7700, 0xB7C1, 0xB681, 0x7640, 0x7200, 0xB2C1, 0xB381,
        0x7340, 0xB101, 0x71C0, 0x7080, 0xB041, 0x5000, 0x90C1, 0x9181, 0x5140, 0x9301, 0x53C0,
        0x5280, 0x9241, 0x9601, 0x56C0, 0x5780, 0x9741, 0x5500, 0x95C1, 0x9481, 0x5440, 0x9C01,
        0x5CC0, 0x5D80, 0x9D41, 0x5F00, 0x9FC1, 0x9E81, 0x5E40, 0x5A00, 0x9AC1, 0x9B81, 0x5B40,
        0x9901, 0x59C0, 0x5880, 0x9841, 0x8801, 0x48C0, 0x4980, 0x8941, 0x4B00, 0x8BC1, 0x8A81,
        0x4A40, 0x4E00, 0x8EC1, 0x8F81, 0x4F40, 0x8D01, 0x4DC0, 0x4C80, 0x8C41, 0x4400, 0x84C1,
        0x8581, 0x4540, 0x8701, 0x47C0, 0x4680, 0x8641, 0x8201, 0x42C0, 0x4380, 0x8341, 0x4100,
        0x81C1, 0x8081, 0x4040,
    ];

    let mut crc_word: u16 = 0xFFFF;
    let mut count = 0;
    while len > 0 {
        let temp = (data[count] as u16 ^ crc_word) as u8;
        count += 1;
        crc_word >>= 8;
        crc_word ^= crc_table[temp as usize];
        len -= 1;
    }
    crc_word
} // End: CRC16

impl ModbusMessage {
    pub fn sent_from_data(mut src: Vec<u8>) -> Self {
        //writing multiple coils or registers not supported yet
        let add = src.remove(0);
        let func = FunctionTypes::new_function_type(src.remove(0));
        let mut start_address: u16 = (src.remove(0) as u16) << 8;
        start_address |= src.remove(0) as u16;

        let mut data_type = DataTypes::None;
        match func {
            FunctionTypes::ReadCoilStatus
            | FunctionTypes::WriteSingleCoil
            | FunctionTypes::WriteMultipleCoils => data_type = DataTypes::Coil,
            FunctionTypes::ReadInputStatus => data_type = DataTypes::Input,

            FunctionTypes::WriteSingleRegister
            | FunctionTypes::WriteMultipleRegisters
            | FunctionTypes::ReadHoldingRegisters
            | FunctionTypes::ReadInputRegisters => data_type = DataTypes::Register,
            _ => {}
        }

        let mut data_vec: Vec<u8> = Vec::new();
        let mut crc_slice: [u8; 2] = [0; 2];
        let mut num_data_points = 0;

        match func {
            FunctionTypes::ReadCoilStatus
            | FunctionTypes::ReadHoldingRegisters
            | FunctionTypes::ReadInputRegisters
            | FunctionTypes::ReadInputStatus => {
                num_data_points = (src.remove(0) as u16) << 8;
                num_data_points |= src.remove(0) as u16;
            }
            FunctionTypes::WriteSingleCoil | FunctionTypes::WriteSingleRegister => {
                data_vec.push(src.remove(0));
                data_vec.push(src.remove(0));
            }
            FunctionTypes::WriteMultipleCoils | FunctionTypes::WriteMultipleRegisters => {}
            _ => {}
        }

        crc_slice[0] = src.remove(0);
        crc_slice[1] = src.remove(0);

        let ret = ModbusMessage {
            start_address: start_address,
            address: add,
            function: func,
            data: data_vec,
            crc: crc_slice,
            num_data_points: num_data_points,
            data_type: data_type,
        };
        println!("sent message: {:?}", ret);
        ret
    }

    pub fn received_from_data(mut message_vec: Vec<u8>) -> Self {
        //writing multiple coils or registers not supported
        let add = message_vec.remove(0);
        let func = FunctionTypes::new_function_type(message_vec.remove(0));
        let mut data_type = DataTypes::None;
        match func {
            FunctionTypes::ReadCoilStatus
            | FunctionTypes::WriteSingleCoil
            | FunctionTypes::WriteMultipleCoils => data_type = DataTypes::Coil,
            FunctionTypes::ReadInputStatus => data_type = DataTypes::Input,

            FunctionTypes::WriteSingleRegister
            | FunctionTypes::WriteMultipleRegisters
            | FunctionTypes::ReadHoldingRegisters
            | FunctionTypes::ReadInputRegisters => data_type = DataTypes::Register,
            _ => {}
        }
        let mut start_address: u16 = 0;
        let mut crc: [u8; 2] = [0; 2];
        let mut data_vec = Vec::new();
        let mut num_data_points = 0;
        match func {
            FunctionTypes::ReadCoilStatus
            | FunctionTypes::ReadHoldingRegisters
            | FunctionTypes::ReadInputRegisters
            | FunctionTypes::ReadInputStatus => {
                let byte_count = message_vec.remove(0);
                for _i in 0..byte_count {
                    data_vec.push(message_vec.remove(0));
                }
            }
            FunctionTypes::WriteSingleCoil | FunctionTypes::WriteSingleRegister => {
                start_address = (message_vec.remove(0) as u16) << 8;
                start_address |= message_vec.remove(0) as u16;
                data_vec.push(message_vec.remove(0));
                data_vec.push(message_vec.remove(0));
            }
            FunctionTypes::WriteMultipleCoils | FunctionTypes::WriteMultipleRegisters => {
                //this isn't supported yet
                start_address = (message_vec.remove(0) as u16) << 8;
                start_address |= message_vec.remove(0) as u16;
                num_data_points = (message_vec.remove(0) as u16) << 8;
                num_data_points |= message_vec.remove(0) as u16;
            }
            _ => {}
        }

        crc[0] = message_vec.remove(0);
        crc[1] = message_vec.remove(0);
        let ret = ModbusMessage {
            start_address: start_address,
            address: add,
            function: func,
            data: data_vec,
            crc: crc,
            num_data_points: num_data_points,
            data_type: data_type,
        };
        print!("received: {:?}", ret);
        ret
    }

    pub fn new_write_message(
        add: u8,
        func: u8,
        start_address: u16,
        num_data_points: u16,
        write_data: Vec<u8>,
    ) -> Self {
        let mut data_type = DataTypes::Coil;
        let function = FunctionTypes::new_function_type(func);
        match function {
            FunctionTypes::ReadCoilStatus
            | FunctionTypes::WriteSingleCoil
            | FunctionTypes::WriteMultipleCoils => data_type = DataTypes::Coil,
            FunctionTypes::ReadInputStatus => data_type = DataTypes::Input,

            FunctionTypes::WriteSingleRegister
            | FunctionTypes::WriteMultipleRegisters
            | FunctionTypes::ReadHoldingRegisters
            | FunctionTypes::ReadInputRegisters => data_type = DataTypes::Register,
            _ => {}
        }

        let mut message = ModbusMessage {
            start_address: start_address,
            address: add,
            function: function,
            data: write_data,
            crc: [0; 2],
            num_data_points: num_data_points,
            data_type: data_type,
        };

        let new_crc = calculate_crc(message.clone());
        message.crc = new_crc;

        message
    }

    pub fn new_read_message(add: u8, func: u8, start_address: u16, num_data_points: u16) -> Self {
        ModbusMessage::new_write_message(add, func, start_address, num_data_points, Vec::new())
    }

    pub fn as_bytes(&mut self) -> Vec<u8> {
        //Currently This does not support writing multiple coils or registers

        let mut data: Vec<u8> = Vec::new();

        data.push(self.address);
        data.push(self.function.clone() as u8);
        data.push((self.start_address >> 8) as u8);
        data.push((self.start_address & 0x0FF) as u8);

        match self.function {
            FunctionTypes::WriteSingleCoil
            | FunctionTypes::WriteSingleRegister
            | FunctionTypes::None => {}
            FunctionTypes::ReadCoilStatus
            | FunctionTypes::ReadHoldingRegisters
            | FunctionTypes::ReadInputRegisters
            | FunctionTypes::ReadInputStatus
            | FunctionTypes::WriteMultipleCoils
            | FunctionTypes::WriteMultipleRegisters => {
                data.push((self.num_data_points >> 8) as u8);
                data.push((self.num_data_points & 0x0FF) as u8);
            }
        }

        match self.function {
            FunctionTypes::WriteSingleCoil
            | FunctionTypes::WriteSingleRegister
            | FunctionTypes::WriteMultipleCoils
            | FunctionTypes::WriteMultipleRegisters => data.append(&mut self.data.clone()),
            _ => {}
        }

        data.extend_from_slice(&self.crc.clone());
        data
    }
}
