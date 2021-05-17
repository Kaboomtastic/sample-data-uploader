use crate::packet::*;
use crate::task::*;
use std::time::Instant;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum DeviceTypes {
    None = 0,
    PowerMeter = 1,
    Bridge = 2,
}

#[derive(Clone, PartialEq)]
pub struct SentMessage {
    pub timestamp: Instant,
    pub address: [u8; 8],
    pub task: Task,
    pub id: u8,
    pub resent: bool,
}

#[derive(Clone)]
pub struct Device {
    pub device_type: DeviceTypes,
    pub address: [u8; 8],
    pub network_address: [u8; 2],
    pub messages_sent: Vec<SentMessage>,
    pub last_heard_from: Instant,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeviceSummary {
    pub device_type: DeviceTypes,
    pub address: [u8; 8],
    pub network_address: [u8; 2],
    pub secs_since_heard_from: u64,
}

pub struct DeviceDB {
    pub devices: Vec<Device>,
    db: sled::Db,
}

impl DeviceTypes {
    pub fn new(val: u8) -> Self {
        match val {
            1 => DeviceTypes::PowerMeter,
            2 => DeviceTypes::Bridge,
            _ => DeviceTypes::None,
        }
    }
}

impl DeviceDB {
    pub fn new() -> Self {
        match sled::open("devices") {
            Ok(db) => {
                let mut devicedb = DeviceDB {
                    devices: Vec::new(),
                    db: db,
                };
                devicedb.db.iter().for_each(|x| {
                    devicedb
                        .devices
                        .push(Device::from_ivec((x.unwrap().1).as_ref()))
                });
                devicedb
            }
            Err(e) => {
                eprintln!("Error Opening Device DB");
                eprintln!("{:}", e);
                std::process::exit(1);
            }
        }
    }

    pub fn add_device(&mut self, device: Device) {
        for d in self.devices.iter() {
            if d.eq(&device) {
                return;
            }
        }

        self.devices.push(device.clone());
        match self.db.insert(device.address, device.clone().to_ivec()) {
            Ok(_t) => {}
            Err(_e) => println!("Failed to add device"),
        }
    }

    pub fn remove_device(&mut self, device: Device) -> bool {
        let mut ret = false;

        for d in self.devices.clone().iter() {
            if d.eq(&device) {
                match self.devices.iter().position(|x| d == x) {
                    Some(p) => {
                        self.devices.remove(p);
                        match self.db.remove(d.address) {
                            Ok(_t) => ret = true,
                            Err(_e) => ret = false,
                        }
                    }
                    None => ret = false,
                }
            }
        }
        ret
    }

    pub fn get_undelivered_packets(&mut self) -> Vec<SentMessage> {
        let mut undelivered = Vec::new();
        self.devices.iter().for_each(|x| {
            x.messages_sent
                .iter()
                .for_each(|y| undelivered.push(y.clone()))
        });
        undelivered
    }

    pub fn clear_resent_message(&mut self, sent_message: SentMessage) {
        for device in &mut self.devices {
            match device
                .messages_sent
                .clone()
                .iter()
                .position(|x| *x == sent_message)
            {
                Some(p) => {
                    device.messages_sent.remove(p);
                    break;
                }
                None => {}
            }
        }
    }

    pub fn add_sent_message(&mut self, task: Task, id: u8, resent: bool) {
        match task.packet.is_broadcast {
            true => {
                for device in self.devices.iter_mut() {
                    if device.device_type == task.device_type {
                        println!(
                            "adding sent message to {:?} at {:X?}",
                            device.device_type.clone(),
                            device.address
                        );
                        device.messages_sent.push(SentMessage {
                            timestamp: Instant::now(),
                            task: task.clone(),
                            id: id,
                            resent: resent,
                            address: device.address,
                        });
                    }
                }
            }
            false => {
                for device in self.devices.iter_mut() {
                    if device.address == task.packet.address {
                        print!(
                            "adding sent message to {:?} at {:X?}",
                            device.device_type.clone(),
                            device.address
                        );
                        device.messages_sent.push(SentMessage {
                            timestamp: Instant::now(),
                            task: task,
                            id: id,
                            resent,
                            address: device.address,
                        });
                        break;
                    }
                }
            }
        }
    }

    pub fn clear_sent_message(&mut self, packet: &mut Packet) -> (DeviceTypes, Packet) {
        match self
            .devices
            .iter()
            .position(|x| packet.address == x.address)
        {
            Some(p) => {
                match packet.frame_type {
                    FrameTypes::TransmitStatus | FrameTypes::RemoteAtResponse => {
                        if packet.delivery_status == 0x00 {
                            println!("packet delivered to {:X?}", self.devices[p].address);
                        } else {
                            println!("packet not delivered");
                            //TODO do something here if packets not delivered
                        }
                        (self.devices[p].device_type.clone(), Packet::new_empty())
                    }

                    FrameTypes::ReceivePacket => {
                        self.devices[p].last_heard_from = Instant::now();

                        let ret_packet;
                        let dev = self.devices[p].device_type.clone();

                        match self.devices[p].device_type {
                            _ => {
                                //for other devices we use the packet identifier
                                let pos = self.devices[p]
                                    .messages_sent
                                    .iter_mut()
                                    .position(|x| x.id == packet.get_packet_identifier())
                                    .unwrap();
                                let ret = self.devices[p].messages_sent.remove(pos);
                                ret_packet = ret.task.packet;
                                println!("cleared sent message")
                            }
                        }

                        Device::remove_identifier(packet, dev.clone()); //we remove the identifier on sent and received so we don't have to worry about it when reading the packet later

                        (dev, ret_packet)
                    }
                    _ => (self.devices[p].device_type.clone(), Packet::new_empty()),
                }
            }
            None => {
                match packet.frame_type {
                    FrameTypes::RemoteAtResponse => {
                        //assumes it's a discovery message but unlikely to be anything else because we do device discovery on startup so we shouldn't be receving messages from unknown devices otherwise
                        let mut data = packet.data.clone();
                        match packet.command {
                            [0x44, 0x44] => {
                                let dtype = DeviceTypes::new(data.pop().unwrap());
                                println!("new {:?}", dtype);
                                //TODO turn data into uint8
                                let dev =
                                    Device::new(dtype.clone(), packet.address, packet.network_address);
                                self.add_device(dev);
                                (dtype, Packet::new_empty())
                            }
                            _ => {
                                (DeviceTypes::None, Packet::new_empty())
                            }
                        }
                    }
                    FrameTypes::TransmitStatus => {
                        if packet.delivery_status == 0x00 {
                            println!("broadcast delivered");
                        } else {
                            println!("broadcast not delivered");
                        }
                        (DeviceTypes::None, Packet::new_empty())
                    }
                    _ => (DeviceTypes::None, Packet::new_empty()),
                }
            }
        }
    }
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}
impl Eq for Device {}

impl Device {
    pub fn remove_identifier(received_packet: &mut Packet, device_type: DeviceTypes) {
        match device_type {
            _ => {
                //The received packet has identifier as first byte
                received_packet.remove_packet_identifier(0);
            }
        }
    }
    pub fn add_identifier(packet: &mut Packet, device_type: DeviceTypes, id: u8) {
        match device_type {
            _ => {
                packet.set_frame_id(id);
                packet.insert_packet_identifer(id);
            }
        }
    }

    pub fn new(d_type: DeviceTypes, address: [u8; 8], network: [u8; 2]) -> Self {
        Device {
            device_type: d_type,
            address: address,
            network_address: network,
            messages_sent: Vec::new(),
            last_heard_from: Instant::now(),
        }
    }

    pub fn to_ivec(&self) -> sled::IVec {
        let mut vec: Vec<u8> = Vec::new();
        vec.push(self.device_type.clone() as u8);
        vec.extend_from_slice(&self.address);
        vec.extend_from_slice(&self.network_address);

        sled::IVec::from(vec)
    }

    pub fn from_ivec(data: &[u8]) -> Self {
        let mut data_vec = Vec::new();
        let mut add_vec: Vec<u8> = Vec::new();
        let mut net_vec: Vec<u8> = Vec::new();

        data_vec.extend_from_slice(data);
        let d_type = DeviceTypes::new(data_vec.remove(0));

        let mut device = Device {
            device_type: d_type,
            address: [0; 8],
            network_address: [0; 2],
            messages_sent: Vec::new(),
            last_heard_from: Instant::now(),
        };

        device
            .address
            .iter()
            .for_each(|_x| add_vec.push(data_vec.remove(0)));
        device
            .network_address
            .iter()
            .for_each(|_x| net_vec.push(data_vec.remove(0)));

        device.address.copy_from_slice(add_vec.as_slice());
        device.network_address.copy_from_slice(net_vec.as_slice());

        device
    }
}
