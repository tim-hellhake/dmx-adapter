/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::adapter::Adapter;
use crate::config;
use crate::device::DmxDevice;
use crate::device_handler::DmxDeviceHandler;
use crate::player::Player;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct DmxAdapter {
    player: Arc<Mutex<Player>>,
}

impl DmxAdapter {
    pub fn new() -> Self {
        DmxAdapter {
            player: Arc::new(Mutex::new(Player::new())),
        }
    }

    pub async fn init(
        &mut self,
        adapter: &mut Arc<Mutex<Adapter>>,
        adapter_config: config::Adapter,
    ) -> Result<(), String> {
        self.player
            .lock()
            .await
            .start(adapter_config.serial_port.as_str())?;

        for device_config in adapter_config.devices {
            println!(
                "Creating device '{}' ({})",
                device_config.title, device_config.id
            );

            let dmx_device = DmxDevice::new(device_config);

            match adapter
                .lock()
                .await
                .add_device(dmx_device.description.clone())
                .await
            {
                Ok(gateway_device) => {
                    let dmx_device_handler = DmxDeviceHandler::new(dmx_device, self.player.clone());
                    gateway_device
                        .lock()
                        .await
                        .set_device_handler(Arc::new(Mutex::new(dmx_device_handler)));
                }
                Err(err) => println!("Could not create device: {}", err),
            }
        }

        Ok(())
    }
}
