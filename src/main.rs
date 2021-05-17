#![feature(string_remove_matches)]

pub mod message;
pub mod packet;
pub mod samples;
pub mod device;
pub mod task;
pub mod modbus;

#[macro_use]
extern crate serde_derive;

use reqwest::*;
use std::thread;
use std::time;
use futures::executor::block_on;
use crate::message::*;
use crate::samples::*;
use serde_urlencoded::*;

async fn get_pulse_samples() {
    match reqwest::get("http://localhost:8080/samples/pulse/100").await {
        Ok(t) => {
            match t.json::<Message>().await {
                Ok(res) => {
                    match res {
                        Message::ErrorMessage(e) => {
                            println!("{}",e);
                        },
                        Message::Samples(samples) => {
                            let mut data: Vec<PulseSample> = Vec::new();
                            let mut keys: Vec<Vec<u8>> = Vec::new();
                            for sample in samples {
                                match sample.1 {
                                    Sample::Pulse(p) => {
                                        data.push(p);
                                        keys.push(sample.0);
                                    }
                                    _ => {}
                                }
                            }
                            let client = reqwest::Client::new();
                            match client.post("http://localhost:8000/samples/pulse").json(&data).send().await {
                                Ok(r) => {
                                    
                                    match client.post("http://localhost:8080/clear-samples/pulse").json(&keys).send().await {
                                        Ok(o) => {
                                            println!("Pulse upload ok")
                                        },
                                        Err(e) => {
                                            println!("{}",e);
                                        }
                                    }
                                },
                                Err(e) => {
                                    println!("{}",e);
                                },
                            }
                        }
                        _ => {
                        }
                    }
                },
                Err(e) => {
                    println!("{:?}", e);
                }
            }
        },
        Err(e) => {
            println!("{:?}", e);
        }
    }
}

async fn get_power_samples_json () {
    match reqwest::get("http://localhost:8080/samples/meter/100").await {
        Ok(t) => {
            match t.json::<Message>().await {
                Ok(res) => {
                    match res {
                        Message::ErrorMessage(e) => {
                            println!("{}",e);
                        },
                        
                        Message::Samples(samples) => {
                            let mut data: Vec<MeterSample> = Vec::new();
                            let mut keys: Vec<Vec<u8>> = Vec::new();
                            for sample in samples {
                                match sample.1 {
                                    Sample::Meter(p) => {
                                        data.push(p);
                                        keys.push(sample.0);
                                    }
                                    _ => {}
                                }
                            }
                            
                            let client = reqwest::Client::new();
                            match client.post("http://localhost:8000/samples/meter").json(&data).send().await {
                                Ok(r) => {
                                    println!("Meter upload ok");
                                    
                                    match client.post("http://localhost:8080/clear-samples/meter").json(&keys).send().await {
                                        Ok(o) => {
                                            println!("Meter upload ok");
                                        },
                                        Err(e) => {
                                            println!("{}",e);
                                        }
                                    }
                                    
                                },
                                Err(e) => {
                                    println!("{}",e);
                                },
                            }
                        }
                        _ => {
                        }
                    }
                },
                Err(e) => {
                    println!("{:?}", e);
                }
            }
        },
        Err(e) => {
            println!("{:?}", e);
        }
    }
}


#[tokio::main]
async fn main() {
    
    loop {
        let future = get_pulse_samples();
        block_on(future);
        let future2 = get_power_samples_json();
        block_on(future2);
        thread::sleep(time::Duration::from_millis(50000));
    }
}
