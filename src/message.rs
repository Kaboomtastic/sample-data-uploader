use crate::device::*;
use crate::packet::*;
use crate::samples::*;
use crate::task::*;
use std::sync::mpsc::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Message {
    ErrorMessage(String),
    Task(Task),
    PeriodicTaskList(Vec<Task>),
    DeviceList(Vec<DeviceSummary>),
    Samples(Vec<(Vec<u8>, Sample)>),
    RequestSamples {
        sample_type: SampleTypes,
        num_samples: u16,
    },
    Packet {
        received: Packet,
        sent: Packet,
        device_type: DeviceTypes,
    },
    ClearSamples {
        sample_type: SampleTypes,
        keys: Vec<Vec<u8>>,
    },
}

pub struct MessageCarrier {
    pub message: Message,
    pub tx: Sender<Message>,
}

impl Message {
    pub fn new_task(task: Task) -> Self {
        Message::Task(task)
    }

    pub fn new_packet(received: Packet, sent: Packet, device_type: DeviceTypes) -> Self {
        Message::Packet {
            received,
            sent,
            device_type,
        }
    }

    pub fn new_device_list(device_list: &Vec<DeviceSummary>) -> Self {
        Message::DeviceList(device_list.clone())
    }

    pub fn new_samples(samples: &Vec<(Vec<u8>, Sample)>) -> Self {
        Message::Samples(samples.clone())
    }

    pub fn new_request_samples(sample_type: SampleTypes, num_samples: u16) -> Self {
        Message::RequestSamples {
            sample_type: sample_type,
            num_samples: num_samples,
        }
    }

    pub fn new_error_message(error_message: String) -> Self {
        Message::ErrorMessage(error_message)
    }
}
