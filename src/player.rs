/**
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use dmx::{self, DmxTransmitter};
use dmx_serial::Error;
use std::sync::{Arc, Mutex};
use std::{thread, time};

pub struct Player {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl Player {
    pub fn new() -> Player {
        return Self {
            buffer: Arc::new(Mutex::new(vec![0; 512])),
        };
    }

    pub fn start(&self, serial_port: &str) -> Result<(), Error> {
        let buffer = Arc::clone(&self.buffer);

        match dmx::open_serial(serial_port) {
            Ok(mut dmx_port) => {
                thread::spawn(move || loop {
                    {
                        match buffer.lock() {
                            Ok(buffer) => match dmx_port.send_dmx_packet(&(*buffer)[..]) {
                                Ok(()) => {}
                                Err(err) => println!("Could not send buffer: {}", err),
                            },
                            Err(err) => println!("Could not lock buffer {}", err),
                        }
                    }
                    thread::sleep(time::Duration::from_millis(50));
                });

                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn set(&self, offset: usize, values: Vec<u8>) -> Result<(), String> {
        match self.buffer.lock() {
            Ok(mut buffer) => {
                for i in 0..values.len() {
                    buffer[i + offset] = values[i];
                }
                return Ok(());
            }
            Err(err) => Err(err.to_string()),
        }
    }
}
