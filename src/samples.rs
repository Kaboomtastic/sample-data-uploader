use crate::modbus;
use crate::modbus::*;
use crate::packet::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Hash, Serialize, Deserialize, PartialEq, Debug)]
pub enum MeterDataTypes {
    Power,
    Volatage,
    Current,
    PowerFactor,
    None,
}

impl MeterDataTypes {
    pub fn new(input: u8) -> Self {
        match input {
            0 => MeterDataTypes::Power,
            1 => MeterDataTypes::Volatage,
            2 => MeterDataTypes::Current,
            3 => MeterDataTypes::PowerFactor,
            _ => MeterDataTypes::None,
        }
    }
}

// All Sample Types need to implement this trait
pub trait DeviceSample {
    fn to_ivec(&self) -> (sled::IVec, sled::IVec);
    fn new_empty() -> Self;
    fn from_ivec(ivec: sled::IVec) -> Self;
    fn new(sent: Packet, received: Packet) -> Self;
}

#[derive(Clone, Hash, Serialize, Deserialize, PartialEq, Debug)]
pub struct MeterSample {
    pub timestamp: u128,
    pub hardware_id: [u8; 8],
    pub data_type: MeterDataTypes,
    pub values: Vec<u16>,
}

#[derive(Clone, Hash, Serialize, Deserialize, PartialEq, Debug)]
pub struct BridgeSample {
    pub timestamp: u128,
    pub hardware_id: [u8; 8],
    pub start_address: u16,
    pub function: modbus::FunctionTypes,
    pub values: Vec<u16>,
    pub device_address: u8,
}

#[derive(Clone, Hash, Serialize, Deserialize, PartialEq, Debug)]
pub struct PulseSample {
    pub timestamp: u128,
    pub hardware_id: [u8; 8],
    pub pulses: [u16; 6],
}

#[derive(Clone, Hash, Serialize, Deserialize, PartialEq, Debug)]
pub enum SampleTypes {
    Meter,
    Bridge,
    Pulse,
    None,
}

#[derive(Clone, Hash, Serialize, Deserialize, PartialEq, Debug)]
pub enum Sample {
    Meter(MeterSample),
    Bridge(BridgeSample),
    Pulse(PulseSample),
    None,
}

pub fn time_as_millis(now: SystemTime) -> u128 {
    now.duration_since(UNIX_EPOCH).unwrap().as_millis()
}

pub fn time_from_millis(millis_since_epoch: u128) -> SystemTime {
    let now = UNIX_EPOCH;
    let duration_since_epoch = Duration::from_millis(millis_since_epoch as u64); // TODO see if i can use the full u128 to make the duration object
    let time = now.checked_add(duration_since_epoch).unwrap();
    time
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

impl DeviceSample for MeterSample {
    fn new_empty() -> Self {
        MeterSample {
            timestamp: time_as_millis(UNIX_EPOCH),
            hardware_id: [0; 8],
            data_type: MeterDataTypes::None,
            values: Vec::new(),
        }
    }

    fn new(sent: Packet, received: Packet) -> Self {
        let _watt = vec!['w' as u8];
        match sent.data {
            _watt => {
                let mut values: Vec<u16> = Vec::new();
                for i in 0..12 {
                    values.push(
                        20 * (((received.data[i * 3] as u16) << 4)
                            + ((received.data[i * 3 + 1] as u16) >> 4)),
                    );
                    values.push(
                        20 * ((((received.data[i * 3 + 1] as u16) & 0x0F) << 8)
                            + (received.data[i * 3 + 2] as u16)),
                    );
                }
                let timestamp = SystemTime::now();
                let hardware_id = received.address;

                MeterSample {
                    timestamp: time_as_millis(timestamp),
                    hardware_id: hardware_id,
                    data_type: MeterDataTypes::Power,
                    values: values,
                }
            }
        }
    }

    fn to_ivec(&self) -> (sled::IVec, sled::IVec) {
        let mut vec: Vec<u8> = Vec::new();
        let timestamp = self.timestamp;
        vec.extend_from_slice(&timestamp.to_ne_bytes());
        vec.extend_from_slice(&self.hardware_id);
        vec.push(self.data_type.clone() as u8);
        let mut u8vec: Vec<u8> = Vec::new();
        for d in self.values.clone() {
            u8vec.push((d >> 8) as u8);
            u8vec.push((d & 0x0FF) as u8);
        }
        vec.extend_from_slice(u8vec.as_slice());

        (
            sled::IVec::from(vec.clone()),
            sled::IVec::from(&calculate_hash(&vec).to_ne_bytes()),
        )
    }

    fn from_ivec(ivec: sled::IVec) -> Self {
        let mut as_vec = Vec::new();
        as_vec.extend_from_slice(ivec.as_ref());
        let mut time_slice: [u8; 16] = [0; 16];
        for i in 0..16 {
            time_slice[i] = as_vec.remove(0);
        }
        let time_as_millis = u128::from_ne_bytes(time_slice);

        let mut hardware_id: [u8; 8] = [0; 8];
        for i in 0..8 {
            hardware_id[i] = as_vec.remove(0);
        }

        let data_type = MeterDataTypes::new(as_vec.remove(0));
        let mut values: Vec<u16> = Vec::new();

        for _i in 0..24 {
            let mut val: u16 = (as_vec.remove(0) as u16) << 8;
            val |= as_vec.remove(0) as u16;
            values.push(val);
        }

        MeterSample {
            timestamp: time_as_millis,
            hardware_id: hardware_id,
            data_type: data_type,
            values: values,
        }
    }
}

impl DeviceSample for PulseSample {
    fn new_empty() -> Self {
        PulseSample {
            timestamp: time_as_millis(UNIX_EPOCH),
            hardware_id: [0; 8],
            pulses: [0; 6],
        }
    }

    fn new(_sent: Packet, received: Packet) -> Self {
        let timestamp = SystemTime::now();
        let hardware_id = received.address;
        let mut ret = PulseSample {
            timestamp: time_as_millis(timestamp),
            hardware_id: hardware_id,
            pulses: [0; 6],
        };
        for i in 0..ret.pulses.len() {
            ret.pulses[i] = (received.data[2 * i] as u16) << 8 | received.data[2 * i + 1] as u16;
        }

        ret
    }

    fn to_ivec(&self) -> (sled::IVec, sled::IVec) {
        let mut vec: Vec<u8> = Vec::new();
        let timestamp = self.timestamp;
        vec.extend_from_slice(&timestamp.to_ne_bytes());
        vec.extend_from_slice(&self.hardware_id);
        for i in 0..self.pulses.len() {
            let val = self.pulses[i];
            vec.push((val >> 8) as u8);
            vec.push((val & 0x0FF) as u8);
        }

        (
            sled::IVec::from(vec.clone()),
            sled::IVec::from(&calculate_hash(&vec).to_ne_bytes()),
        )
    }

    fn from_ivec(ivec: sled::IVec) -> Self {
        let mut as_vec = Vec::new();
        as_vec.extend_from_slice(ivec.as_ref());
        let mut time_slice: [u8; 16] = [0; 16];
        for i in 0..16 {
            time_slice[i] = as_vec.remove(0);
        }
        let time_as_millis = u128::from_ne_bytes(time_slice);

        let mut hardware_id: [u8; 8] = [0; 8];
        for i in 0..8 {
            hardware_id[i] = as_vec.remove(0);
        }

        let mut ret = PulseSample {
            timestamp: time_as_millis,
            hardware_id: hardware_id,
            pulses: [0; 6],
        };

        for i in 0..ret.pulses.len() {
            ret.pulses[i] = (as_vec.remove(0) as u16) << 8;
            ret.pulses[i] |= as_vec.remove(0) as u16;
        }
        ret
    }
}

impl DeviceSample for BridgeSample {
    fn new_empty() -> Self {
        BridgeSample {
            timestamp: time_as_millis(UNIX_EPOCH),
            hardware_id: [0; 8],
            start_address: 0,
            function: modbus::FunctionTypes::None,
            values: Vec::new(),
            device_address: 0,
        }
    }
    fn new(sent: Packet, received: Packet) -> Self {
        let received_modbus = ModbusMessage::received_from_data(received.data);
        let sent_modbus = ModbusMessage::sent_from_data(sent.data);

        let timestamp = SystemTime::now();
        let hardware_id = received.address;

        let mut data_points: Vec<u16> = Vec::new();

        match sent_modbus.function {
            FunctionTypes::ReadCoilStatus
            | FunctionTypes::ReadHoldingRegisters
            | FunctionTypes::ReadInputRegisters
            | FunctionTypes::ReadInputStatus => match received_modbus.data_type {
                DataTypes::Coil | DataTypes::Input => {
                    let mut byte_num = 0;
                    for _i in 0..sent_modbus.num_data_points {
                        let result = (received_modbus.data[byte_num]
                            & (0x01 << (sent_modbus.num_data_points % 8)))
                            >> sent_modbus.num_data_points % 8;
                        data_points.push(result as u16);
                        if sent_modbus.num_data_points % 8 == 7 {
                            byte_num += 1;
                        }
                    }
                }
                DataTypes::Register => {
                    for i in 0..sent_modbus.num_data_points {
                        let mut result = (received_modbus.data[(i * 2) as usize] as u16) << 8;
                        result |= (received_modbus.data[(i * 2 + 1) as usize]) as u16;
                        data_points.push(result);
                    }
                }
                DataTypes::None => {}
            },
            FunctionTypes::WriteSingleCoil | FunctionTypes::WriteSingleRegister => {
                match sent_modbus.data_type {
                    DataTypes::Coil | DataTypes::Input => {
                        if sent_modbus.data[0] == 0xFF {
                            data_points.push(1);
                        } else {
                            data_points.push(0);
                        }
                    }
                    DataTypes::Register => {
                        let result =
                            ((sent_modbus.data[0] as u16) << 8) | (sent_modbus.data[1] as u16);
                        data_points.push(result);
                    }
                    DataTypes::None => {}
                }
            }
            FunctionTypes::WriteMultipleCoils | FunctionTypes::WriteMultipleRegisters => { //writing multiple not currently supported
            }
            _ => {}
        }
        let mut write = false;
        match sent_modbus.function {
            FunctionTypes::ReadCoilStatus
            | FunctionTypes::ReadHoldingRegisters
            | FunctionTypes::ReadInputRegisters
            | FunctionTypes::ReadInputStatus => write = false,
            FunctionTypes::WriteSingleCoil
            | FunctionTypes::WriteSingleRegister
            | FunctionTypes::WriteMultipleCoils
            | FunctionTypes::WriteMultipleRegisters => write = true,
            _ => {}
        }
        let data_type = sent_modbus.data_type;

        BridgeSample {
            timestamp: time_as_millis(timestamp),
            hardware_id: hardware_id,
            start_address: sent_modbus.start_address,
            function: sent_modbus.function,
            values: data_points,
            device_address: sent_modbus.address,
        }
    }

    fn to_ivec(&self) -> (sled::IVec, sled::IVec) {
        let mut vec: Vec<u8> = Vec::new();
        let timestamp = self.timestamp;
        vec.extend_from_slice(&timestamp.to_ne_bytes());
        vec.extend_from_slice(&self.hardware_id);
        vec.push(self.device_address);
        vec.push((self.start_address >> 8) as u8);
        vec.push((self.start_address & 0x0FF) as u8);
        vec.push(self.function.clone() as u8);
        for v in self.values.clone() {
            vec.push((v >> 8) as u8);
            vec.push((v & 0x0FF) as u8);
        }
        (
            sled::IVec::from(vec.clone()),
            sled::IVec::from(&calculate_hash(&vec).to_ne_bytes()),
        )
    }

    fn from_ivec(ivec: sled::IVec) -> Self {
        let mut as_vec = Vec::new();
        as_vec.extend_from_slice(ivec.as_ref());
        let mut time_slice: [u8; 16] = [0; 16];
        for i in 0..16 {
            time_slice[i] = as_vec.remove(0);
        }
        let time_as_millis = u128::from_ne_bytes(time_slice);
        let mut hardware_id: [u8; 8] = [0; 8];
        for i in 0..8 {
            hardware_id[i] = as_vec.remove(0);
        }
        let device_address = as_vec.remove(0);
        let mut start_address = (as_vec.remove(0) as u16) << 8;
        start_address |= as_vec.remove(0) as u16;
        let function = FunctionTypes::new(as_vec.remove(0));

        let mut values = Vec::new();
        while as_vec.is_empty() == false {
            let mut val = (as_vec.remove(0) as u16) << 8;
            val |= as_vec.remove(0) as u16;
            values.push(val);
        }

        BridgeSample {
            timestamp: time_as_millis,
            hardware_id: hardware_id,
            start_address: start_address,
            function: function,
            values: values,
            device_address: device_address,
        }
    }
}
