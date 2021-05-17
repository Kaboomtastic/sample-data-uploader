#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FrameTypes {
    TransmitRequest = 0x10,
    TransmitStatus = 0x8B,
    ReceivePacket = 0x90,
    RemoteAtRequest = 0x17,
    RemoteAtResponse = 0x97,
    LocalAtCommand = 0x08,
    LocalAtResponse = 0x88,
    None = 0x00,
}

impl FrameTypes {
    fn new_frame_type(val: u8) -> Self {
        match val {
            0x10 => FrameTypes::TransmitRequest,
            0x8B => FrameTypes::TransmitStatus,
            0x90 => FrameTypes::ReceivePacket,
            0x17 => FrameTypes::RemoteAtRequest,
            0x97 => FrameTypes::RemoteAtResponse,
            0x08 => FrameTypes::LocalAtCommand,
            0x88 => FrameTypes::LocalAtResponse,
            _ => FrameTypes::None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Packet {
    pub is_broadcast: bool,
    pub length: u16,
    pub frame_type: FrameTypes,
    pub frame_id: u8,
    pub address: [u8; 8],
    pub network_address: [u8; 2],
    pub options: u8,
    pub delivery_status: u8,
    pub data: Vec<u8>,
    pub checksum: u8,
    pub retry_count: u8,
    pub discovery_status: u8,
    pub broadcast_radius: u8,
    pub command: [u8; 2],
    pub command_status: u8,
}

pub fn calculate_checksum(packet: Packet) -> u8 {
    let mut sum: u8 = 0;
    sum += packet.frame_type as u8;
    match packet.frame_type {
        FrameTypes::TransmitRequest | FrameTypes::RemoteAtRequest => {
            sum = sum.overflowing_add(packet.frame_id).0;
            packet
                .address
                .iter()
                .for_each(|x| sum = sum.overflowing_add(*x).0);
            packet
                .network_address
                .iter()
                .for_each(|x| sum = sum.overflowing_add(*x).0);
            sum = sum.overflowing_add(packet.options).0;
            packet
                .data
                .iter()
                .for_each(|x| sum = sum.overflowing_add(*x).0);
            if packet.frame_type == FrameTypes::RemoteAtRequest {
                packet
                    .command
                    .iter()
                    .for_each(|x| sum = sum.overflowing_add(*x).0);
            }
        }
        FrameTypes::TransmitStatus => {
            sum = sum.overflowing_add(packet.frame_id).0;
            packet
                .network_address
                .iter()
                .for_each(|x| sum = sum.overflowing_add(*x).0);
            sum = sum.overflowing_add(packet.retry_count).0;
            sum = sum.overflowing_add(packet.delivery_status).0;
            sum = sum.overflowing_add(packet.discovery_status).0;
        }
        FrameTypes::ReceivePacket => {
            packet
                .address
                .iter()
                .for_each(|x| sum = sum.overflowing_add(*x).0);
            packet
                .network_address
                .iter()
                .for_each(|x| sum = sum.overflowing_add(*x).0);
            sum = sum.overflowing_add(packet.options).0;
            packet
                .data
                .iter()
                .for_each(|x| sum = sum.overflowing_add(*x).0);
        }
        FrameTypes::RemoteAtResponse => {
            sum = sum.overflowing_add(packet.frame_id).0;
            packet
                .address
                .iter()
                .for_each(|x| sum = sum.overflowing_add(*x).0);
            packet
                .network_address
                .iter()
                .for_each(|x| sum = sum.overflowing_add(*x).0);
            packet
                .data
                .iter()
                .for_each(|x| sum = sum.overflowing_add(*x).0);

            sum = sum.overflowing_add(packet.options).0;

            packet
                .command
                .iter()
                .for_each(|x| sum = sum.overflowing_add(*x).0);
        }
        FrameTypes::LocalAtCommand | FrameTypes::LocalAtResponse => {
            sum = sum.overflowing_add(packet.frame_id).0;
            packet
                .data
                .iter()
                .for_each(|x| sum = sum.overflowing_add(*x).0);
        }
        _ => sum = 0, //other frame types currently unsupported
    };
    255 - sum
}

impl Packet {
    pub fn new_empty() -> Self {
        Packet {
            is_broadcast: false,
            length: 0x00,
            frame_type: FrameTypes::None,
            frame_id: 0x00,
            address: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            network_address: [0x00, 0x00],
            options: 0x00,
            delivery_status: 0x00,
            data: Vec::new(),
            checksum: 0x00,
            discovery_status: 0x00,
            retry_count: 0x00,
            broadcast_radius: 0x00,
            command: [0x00, 0x00],
            command_status: 0x00,
        }
    }

    pub fn new_broadcast(data: &[u8]) -> Self {
        let mut temp_length: u16 = 14;
        temp_length += data.len() as u16;
        let mut data_vec: Vec<u8> = Vec::new();
        data_vec.extend_from_slice(data);

        let mut packet = Packet {
            is_broadcast: true,
            length: temp_length,
            frame_type: FrameTypes::TransmitRequest,
            frame_id: 0x00,
            address: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF],
            network_address: [0xFF, 0xFE],
            options: 0x00,
            delivery_status: 0x00,
            data: data_vec,
            checksum: 0x00,
            discovery_status: 0x00,
            retry_count: 0x00,
            broadcast_radius: 0x00,
            command: [0x00, 0x00],
            command_status: 0x00,
        };
        packet.checksum = calculate_checksum(packet.clone());
        packet
    }

    pub fn new_transmit(dest: &[u8; 8], data: &[u8]) -> Self {
        let mut temp_length: u16 = 14;
        temp_length += data.len() as u16;
        let mut data_vec: Vec<u8> = Vec::new();
        data_vec.extend_from_slice(data);

        let mut packet = Packet {
            is_broadcast: false,
            length: temp_length,
            frame_type: FrameTypes::TransmitRequest,
            frame_id: 0x00,
            address: dest.clone(),
            network_address: [0xFF, 0xFE],
            options: 0x00,
            delivery_status: 0x00,
            data: data_vec,
            checksum: 0x00,
            discovery_status: 0x00,
            retry_count: 0x00,
            broadcast_radius: 0x00,
            command: [0x00, 0x00],
            command_status: 0x00,
        };
        packet.checksum = calculate_checksum(packet.clone());

        packet
    }

    pub fn new_remote_at(dest: &[u8; 8], id: u8, command: u16, param: &[u8]) -> Self {
        let mut temp_length: u16 = 15;
        temp_length += param.len() as u16;

        let mut data_vec: Vec<u8> = Vec::new();
        data_vec.extend_from_slice(param);

        let command_temp: [u8; 2] = [(command >> 8) as u8, (command & 0x0FF) as u8];

        let mut packet = Packet {
            is_broadcast: false,
            length: temp_length,
            frame_type: FrameTypes::RemoteAtRequest,
            frame_id: id,
            address: dest.clone(),
            network_address: [0xFF, 0xFE],
            options: 0x00,
            delivery_status: 0x00,
            data: data_vec,
            checksum: 0x00,
            discovery_status: 0x00,
            retry_count: 0x00,
            broadcast_radius: 0x00,
            command: command_temp.clone(),
            command_status: 0x00,
        };

        packet.checksum = calculate_checksum(packet.clone());

        packet
    }

    pub fn new_local_at(id: u8, command: u16, param: &[u8]) -> Self {
        let mut temp_length: u16 = 8;
        temp_length += param.len() as u16;

        let mut data_vec: Vec<u8> = Vec::new();
        data_vec.extend_from_slice(param);

        let command_temp: [u8; 2] = [(command >> 8) as u8, (command & 0x0FF) as u8];

        let mut packet = Packet {
            is_broadcast: false,
            length: temp_length,
            frame_type: FrameTypes::LocalAtCommand,
            frame_id: id,
            address: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            network_address: [0x00, 0x00],
            options: 0x00,
            delivery_status: 0x00,
            data: data_vec,
            checksum: 0x00,
            discovery_status: 0x00,
            retry_count: 0x00,
            broadcast_radius: 0x00,
            command: command_temp.clone(),
            command_status: 0x00,
        };

        packet.checksum = calculate_checksum(packet.clone());

        packet
    }

    pub fn set_frame_id(&mut self, identifier: u8) {
        self.frame_id = identifier;
        self.checksum = calculate_checksum(self.clone());
    }

    pub fn insert_packet_identifer(&mut self, identifier: u8) {
        self.data.insert(1, identifier);
        self.length += 1;
        self.checksum = calculate_checksum(self.clone());
    }

    pub fn get_packet_identifier(&self) -> u8 {
        match self.frame_type {
            FrameTypes::TransmitStatus => self.frame_id,
            FrameTypes::TransmitRequest => self.data[1],
            FrameTypes::ReceivePacket => self.data[0],
            FrameTypes::RemoteAtRequest => 255,
            FrameTypes::RemoteAtResponse => 255,
            FrameTypes::LocalAtCommand => 255,
            FrameTypes::LocalAtResponse => 255,
            _ => 0,
        }
    }

    pub fn remove_packet_identifier(&mut self, pos: usize) -> u8 {
        self.data.remove(pos)
    }

    pub fn from_data(raw: &mut Vec<u8>) -> Result<Self, String> {
        match raw.len() {
            d if d < 7 => Err(format!("not enough data")),
            _ => match raw.remove(0) {
                0x7E => {
                    let mut packet = Packet::new_empty();

                    let len_high = raw.remove(0);
                    let len_low = raw.remove(0);
                    packet.length = ((len_high as u16) << 8) | (len_low as u16);
                    let mut length_remaining = packet.length;

                    packet.frame_type = FrameTypes::new_frame_type(raw.remove(0));
                    length_remaining -= 1;

                    match packet.frame_type {
                        FrameTypes::ReceivePacket => {
                            let mut address: Vec<u8> = Vec::new();
                            packet
                                .address
                                .iter()
                                .for_each(|_x| address.push(raw.remove(0)));
                            packet.address.copy_from_slice(address.as_slice());
                            length_remaining -= packet.address.len() as u16;

                            let mut network_address: Vec<u8> = Vec::new();
                            packet
                                .network_address
                                .iter()
                                .for_each(|_x| network_address.push(raw.remove(0)));
                            packet
                                .network_address
                                .copy_from_slice(network_address.as_slice());
                            length_remaining -= packet.network_address.len() as u16;

                            packet.options = raw.remove(0);
                            length_remaining -= 1;

                            for _i in 0..length_remaining {
                                packet.data.push(raw.remove(0));
                            }
                            packet.checksum = raw.remove(0);
                        }
                        FrameTypes::RemoteAtResponse => {
                            packet.frame_id = raw.remove(0);
                            length_remaining -= 1;

                            let mut address: Vec<u8> = Vec::new();
                            packet
                                .address
                                .iter()
                                .for_each(|_x| address.push(raw.remove(0)));
                            packet.address.copy_from_slice(address.as_slice());
                            length_remaining -= packet.address.len() as u16;

                            let mut network_address: Vec<u8> = Vec::new();
                            packet
                                .network_address
                                .iter()
                                .for_each(|_x| network_address.push(raw.remove(0)));
                            packet
                                .network_address
                                .copy_from_slice(network_address.as_slice());
                            length_remaining -= packet.network_address.len() as u16;

                            let mut command: Vec<u8> = Vec::new();
                            packet
                                .command
                                .iter()
                                .for_each(|_x| command.push(raw.remove(0)));
                            packet.command.copy_from_slice(command.as_slice());
                            length_remaining -= packet.command.len() as u16;

                            packet.options = raw.remove(0);
                            length_remaining -= 1;

                            for _i in 0..length_remaining {
                                packet.data.push(raw.remove(0));
                            }
                            packet.checksum = raw.remove(0);
                        }
                        FrameTypes::TransmitStatus => {
                            packet.frame_id = raw.remove(0);
                            let mut network_address: Vec<u8> = Vec::new();
                            packet
                                .network_address
                                .iter()
                                .for_each(|_x| network_address.push(raw.remove(0)));
                            packet
                                .network_address
                                .copy_from_slice(network_address.as_slice());
                            packet.retry_count = raw.remove(0);
                            packet.delivery_status = raw.remove(0);
                            packet.discovery_status = raw.remove(0);
                            packet.checksum = raw.remove(0);
                        }
                        FrameTypes::LocalAtResponse => {
                            packet.frame_id = raw.remove(0);
                            length_remaining -= 1;

                            let mut command: Vec<u8> = Vec::new();
                            packet
                                .command
                                .iter()
                                .for_each(|_x| command.push(raw.remove(0)));
                            packet.command.copy_from_slice(command.as_slice());
                            length_remaining -= packet.command.len() as u16;

                            packet.command_status = raw.remove(0);

                            for _i in 0..length_remaining {
                                packet.data.push(raw.remove(0));
                            }

                            packet.checksum = raw.remove(0);
                        }
                        FrameTypes::TransmitRequest => {
                            packet.frame_id = raw.remove(0);
                            length_remaining -= 1;

                            let mut address: Vec<u8> = Vec::new();
                            packet
                                .address
                                .iter()
                                .for_each(|_x| address.push(raw.remove(0)));
                            packet.address.copy_from_slice(address.as_slice());
                            length_remaining -= packet.address.len() as u16;

                            let mut network_address: Vec<u8> = Vec::new();
                            packet
                                .network_address
                                .iter()
                                .for_each(|_x| network_address.push(raw.remove(0)));
                            packet
                                .network_address
                                .copy_from_slice(network_address.as_slice());
                            length_remaining -= packet.network_address.len() as u16;

                            packet.broadcast_radius = raw.remove(0);
                            length_remaining -= 1;

                            packet.options = raw.remove(0);
                            length_remaining -= 1;

                            for _i in 0..length_remaining {
                                packet.data.push(raw.remove(0));
                            }

                            packet.checksum = raw.remove(0);
                        }
                        _ => {}
                    }
                    if packet.address == [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF] {
                        packet.is_broadcast = true;
                    }
                    match packet.is_valid() {
                        false => Err(format!("Invalid Checksum")),
                        true => Ok(packet),
                    }
                }
                _ => Err(format!("Invalid Start Byte")),
            },
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        match self.frame_type {
            FrameTypes::TransmitRequest => {
                bytes.push(0x7E);
                let len_high: u8 = (self.length >> 8) as u8;
                let len_low: u8 = (self.length & 0x00FF) as u8;
                bytes.push(len_high);
                bytes.push(len_low);
                bytes.push(self.frame_type as u8);
                bytes.push(self.frame_id);
                self.address.iter().for_each(|x| bytes.push(*x));
                self.network_address.iter().for_each(|x| bytes.push(*x));
                bytes.push(self.broadcast_radius);
                bytes.push(self.options);
                bytes.append(&mut self.data.clone());
                bytes.push(self.checksum);
            }
            FrameTypes::RemoteAtRequest => {
                bytes.push(0x7E);
                let len_high: u8 = (self.length >> 8) as u8;
                let len_low: u8 = (self.length & 0x00FF) as u8;
                bytes.push(len_high);
                bytes.push(len_low);
                bytes.push(self.frame_type as u8);
                bytes.push(self.frame_id);
                self.address.iter().for_each(|x| bytes.push(*x));
                self.network_address.iter().for_each(|x| bytes.push(*x));
                bytes.push(self.options);
                self.command.iter().for_each(|x| bytes.push(*x));
                bytes.append(&mut self.data.clone());
                bytes.push(self.checksum);
            }
            FrameTypes::LocalAtCommand => {
                bytes.push(0x7E);
                let len_high: u8 = (self.length >> 8) as u8;
                let len_low: u8 = (self.length & 0x00FF) as u8;
                bytes.push(len_high);
                bytes.push(len_low);
                bytes.push(self.frame_type as u8);
                bytes.push(self.frame_id);
                self.command.iter().for_each(|x| bytes.push(*x));
                bytes.append(&mut self.data.clone());
                bytes.push(self.checksum);
            }
            _ => bytes.push(0x00), //need to support more frame types
                                   //FrameTypes::RemoteAtRequest =>
        }
        bytes
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn is_valid(&self) -> bool {
        self.checksum == calculate_checksum(self.clone())
    }
}
