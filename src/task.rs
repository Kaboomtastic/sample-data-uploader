use crate::device::DeviceTypes;
use crate::packet::*;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum TaskTypes {
    Periodic,
    Single,
    RemovePeriodic,
    PausePeriodic,
    GetPeriodic,
    GetDevices,
    AddDevice,
    Discover,
    None,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Task {
    pub task_type: TaskTypes,
    pub packet: Packet,
    pub min: usize,
    pub sec: usize,
    pub device_type: DeviceTypes,
}

impl TaskTypes {
    pub fn new_task_type(val: u8) -> Self {
        match val {
            0 => TaskTypes::Periodic,
            1 => TaskTypes::Single,
            2 => TaskTypes::RemovePeriodic,
            3 => TaskTypes::PausePeriodic,
            _ => TaskTypes::None,
        }
    }
}

impl Eq for Task {}

impl Task {
    pub fn new(
        packet: Packet,
        task_type: TaskTypes,
        min: usize,
        sec: usize,
        device: DeviceTypes,
    ) -> Self {
        Task {
            task_type: task_type,
            packet: packet,
            min: min,
            sec: sec,
            device_type: device,
        }
    }
    pub fn to_vec(&mut self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        vec.push(self.task_type.clone() as u8);
        vec.extend_from_slice(&self.min.to_ne_bytes());
        vec.extend_from_slice(&self.sec.to_ne_bytes());
        vec.push(self.device_type.clone() as u8);
        vec.append(&mut self.packet.as_bytes());
        vec
    }
    pub fn from_vec(mut data: Vec<u8>) -> Self {
        let t_type = TaskTypes::new_task_type(data.remove(0));

        let mut min: Vec<u8> = Vec::new();
        let mut sec: Vec<u8> = Vec::new();

        for _i in 0..std::mem::size_of::<usize>() {
            min.push(data.remove(0));
        }
        for _i in 0..std::mem::size_of::<usize>() {
            sec.push(data.remove(0));
        }
        let dev = DeviceTypes::new(data.remove(0));
        let pack = Packet::from_data(&mut data.clone()).unwrap();
        let mut minslice: [u8; 8] = [0; 8];
        let mut secslice: [u8; 8] = [0; 8];

        minslice.copy_from_slice(min.as_slice());
        secslice.copy_from_slice(sec.as_slice());

        Task {
            task_type: t_type,
            packet: pack,
            min: usize::from_ne_bytes(minslice),
            sec: usize::from_ne_bytes(secslice),
            device_type: dev,
        }
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.packet.data == other.packet.data && self.packet.address == other.packet.address
    }
}
