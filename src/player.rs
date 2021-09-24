/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use dmx::{self, DmxTransmitter};
use dmx_serial::SystemPort;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::{thread, time};

pub struct Player {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl Player {
    pub fn new() -> Player {
        Self {
            buffer: Arc::new(Mutex::new(vec![0; 512])),
        }
    }

    pub fn start(&self, serial_port: &str) -> Result<(), String> {
        let buffer = self.buffer.clone();

        let mut dmx_port = dmx::open_serial(serial_port)
            .map_err(|err| format!("Could not open serial port: {}", err))?;

        thread::spawn(move || loop {
            if let Err(err) = Player::send_buffer(&mut dmx_port, &buffer) {
                log::error!("Could not send buffer: {}", err);
            }
            thread::sleep(time::Duration::from_millis(50));
        });

        Ok(())
    }

    fn send_buffer(
        dmx_port: &mut SystemPort,
        buffer: &Arc<Mutex<Vec<u8>>>,
    ) -> Result<(), Box<dyn Error>> {
        let buffer = buffer
            .lock()
            .map_err(|err| format!("Could not lock buffer : {}", err))?;

        dmx_port
            .send_dmx_packet(&(*buffer)[..])
            .map_err(|err| err)?;

        Ok(())
    }

    pub async fn set(&self, offset: usize, values: Vec<u8>) -> Result<(), String> {
        let buffer = Arc::clone(&self.buffer);

        tokio::task::spawn_blocking(move || {
            let buffer = buffer
                .lock()
                .map_err(|err| format!("Could not lock buffer: {}", err));

            match buffer {
                Ok(mut buffer) => {
                    for i in 0..values.len() {
                        buffer[i + offset] = values[i];
                    }

                    Ok(())
                }
                Err(err) => Err(err),
            }
        })
        .await
        .map_err(|err| format!("Could set byte: {}", err))?
    }
}
