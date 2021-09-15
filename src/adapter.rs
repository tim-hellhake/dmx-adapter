/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::adapter::{Adapter, AdapterHandle};
use crate::config::Adapter as AdapterConfig;
use crate::device::DmxDevice;
use crate::player::Player;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct DmxAdapter {
    adapter_handle: AdapterHandle,
    player: Arc<Mutex<Player>>,
}

impl DmxAdapter {
    pub fn new(adapter_handle: AdapterHandle) -> Self {
        DmxAdapter {
            adapter_handle,
            player: Arc::new(Mutex::new(Player::new())),
        }
    }

    pub async fn init(&mut self, adapter_config: AdapterConfig) -> Result<(), String> {
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

            let player = self.player.clone();

            if let Err(err) = self
                .adapter_handle
                .add_device(description, |device| {
                    DmxDevice::new(device_config, device, player)
                })
                .await
            {
                log::error!("Could not create device: {}", err)
            }
        }

        Ok(())
    }
}

impl Adapter for DmxAdapter {
    fn get_adapter_handle(&self) -> &AdapterHandle {
        &self.adapter_handle
    }
}
