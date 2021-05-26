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
use std::env;
use crate::message::*;
use crate::samples::*;

struct Config {
    ingest_url_base: String,
    dc_url_base: String,
    num_samples: u16,
    delay: u64,
}

async fn clear_samples(url: &str, keys: Vec<Vec<u8>>, client: &Client) {
    let mut done = false;

    while !done {
        match client.post(url).json(&keys).send().await {
            Ok(o) => {
                println!("got clear response");
                match o.json::<Message>().await {
                    Ok(r) => {
                        println!("clear response deserialized");
                        match r {
                            Message::ErrorMessage(s) => {
                                match s.as_str() {
                                    "Samples Removed" => {
                                        done = true;
                                    },
                                    _ => {
                                        eprintln!("Unexpected message {}", s);
                                    }
                                }
                            },
                            _ => {
                                eprintln!("Unexpected response type {:?}", r);
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            },
            Err(e) => {
                eprintln!("{}",e);
            }
        }
    }
}

async fn get_pulse_samples(config: &Config, client: &Client) {
    let req = format!("{}/samples/pulse/{}", config.dc_url_base, config.num_samples);
    
    match client.get(req).send().await {
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
                            if samples.len() == 0 {
                                eprintln!("Error: expected samples but found none");
                                return
                            }
                            for sample in samples {
                                match sample.1 {
                                    Sample::Pulse(p) => {
                                        data.push(p);
                                        keys.push(sample.0);
                                    }
                                    _ => {
                                        eprintln!("Error: Unexpected sample type");
                                    }
                                }
                            }

                            let req = format!("{}/samples/pulse", config.ingest_url_base);
                            match client.post(req).json(&data).send().await {
                                Ok(r) => {
                                    match r.status() {
                                        StatusCode::OK => {
                                            let req = format!("{}/clear-samples/pulse", config.dc_url_base);
                                            clear_samples(&req, keys, client).await;
                                        },
                                        _ => {
                                            eprintln!("{}", r.status());
                                        }
                                    }
                                },
                                Err(e) => {
                                    eprintln!("{}",e);
                                },
                            }
                        },
                        _ => {
                            eprintln!("Error: Unexpected response type");
                        }
                    }
                },
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        },
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}

async fn get_power_samples_json (config: &Config, client: &Client) {
    let req = format!("{}/samples/meter/{}", config.dc_url_base, config.num_samples);
    match client.get(req).send().await {
        Ok(t) => {
            match t.json::<Message>().await {
                Ok(res) => {
                    match res {
                        Message::ErrorMessage(e) => {
                            eprintln!("{}",e);
                        },
                        Message::Samples(samples) => {
                            let mut data: Vec<MeterSample> = Vec::new();
                            let mut keys: Vec<Vec<u8>> = Vec::new();
                            if samples.len() == 0 {
                                eprintln!("Error: expected samples but found none");
                                return
                            }
                            for sample in samples {
                                match sample.1 {
                                    Sample::Meter(p) => {
                                        data.push(p);
                                        keys.push(sample.0);
                                    }
                                    _ => {
                                        eprintln!("Error: Unexpected sample type");
                                    }
                                }
                            }
                            
                            let req = format!("{}/samples/meter", config.ingest_url_base);
                            match client.post(req).json(&data).send().await {
                                Ok(r) => {
                                    match r.status() {
                                        StatusCode::OK => {
                                            let req = format!("{}/clear-samples/meter", config.dc_url_base);
                                            clear_samples(&req, keys, &client).await;
                                        },
                                        _ => {
                                            eprintln!("{}", r.status());
                                        }
                                    }
                                },
                                Err(e) => {
                                    eprintln!("{}",e);
                                },
                            }
                        }
                        _ => {
                            eprintln!("Error: Unexpected response type");
                        }
                    }
                },
                Err(e) => {
                    eprintln!("{:?}", e);
                }
            }
        },
        Err(e) => {
            eprintln!("{:?}", e);
        }
    }
}


#[tokio::main]
async fn main() {
    
    let mut config = Config {
        ingest_url_base: "".to_string(),
        dc_url_base: "".to_string(),
        num_samples: 0,
        delay: 0,
    };

    match env::var("SAMPLE_INGEST_URL") {
        Ok(val) => {
            config.ingest_url_base = val;
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    match env::var("DATACOLLECTOR_URL") {
        Ok(val) => {
            config.dc_url_base = val;
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    match env::var("NUM_SAMPLES") {
        Ok(val) => {
            match val.parse::<u16>() {
                Ok(v) => {
                    config.num_samples = v;
                },
                Err(e) => {
                    eprintln!("Error {}", e);
                    std::process::exit(1);
                }
            }
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    match env::var("UPLOAD_DELAY") {
        Ok(val) => {
            match val.parse::<u64>() {
                Ok(v) => {
                    config.delay = v;
                },
                Err(e) => {
                    eprintln!("Error {}", e);
                    std::process::exit(1);
                }
            }
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    let client = reqwest::Client::builder().connection_verbose(true)
    .connect_timeout(time::Duration::from_millis(500))
    .timeout(time::Duration::from_millis(2000))
    .pool_idle_timeout(Some(time::Duration::from_secs(10)))
    .pool_max_idle_per_host(3)
    .build().unwrap();
    loop {
        println!("starting upload...");
        get_pulse_samples(&config, &client).await;
        get_power_samples_json(&config, &client).await;
        thread::sleep(time::Duration::from_millis(config.delay));
    }
}
