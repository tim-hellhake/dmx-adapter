/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::adapter::Adapter;
use crate::config;
use crate::device::DmxDevice;
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
            log::debug!(
                "Creating device '{}' ({})",
                device_config.title,
                device_config.id
            );

            let description = DmxDevice::build_description(&device_config);

            if let Err(err) = adapter
                .lock()
                .await
                .add_device(description, |device| {
                    DmxDevice::new(device_config, device, self.player.clone())
                })
                .await
            {
                log::error!("Could not create device: {}", err)
            }
        }

        Ok(())
    }
}
