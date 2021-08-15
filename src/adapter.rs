/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::adapter::Adapter;
use crate::api::device::Device;
use crate::config;
use crate::device::DmxDevice;
use crate::player::Player;
use serde_json::value::Value;
use std::collections::HashMap;

pub struct DmxAdapter {
    player: Player,
    devices: HashMap<String, (DmxDevice, Device)>,
}

impl DmxAdapter {
    pub fn new() -> Self {
        DmxAdapter {
            player: Player::new(),
            devices: HashMap::new(),
        }
    }

    pub async fn init(
        &mut self,
        adapter: &Adapter,
        adapter_config: config::Adapter,
    ) -> Result<(), String> {
        match self.player.start(adapter_config.serial_port.as_str()) {
            Ok(()) => {
                for device_config in adapter_config.devices {
                    println!(
                        "Creating device '{}' ({})",
                        device_config.title,
                        device_config.id.as_ref().unwrap_or(&String::from(""))
                    );

                    let dmx_device = DmxDevice::new(device_config);

                    match adapter.add_device(dmx_device.description.clone()).await {
                        Ok(gateway_device) => {
                            let id = dmx_device.description.id.clone();
                            self.devices
                                .insert(id.clone(), (dmx_device, gateway_device));
                        }
                        Err(err) => println!("Could not create device: {}", err),
                    }
                }
                Ok(())
            }
            Err(err) => Err(err.to_string()),
        }
    }

    pub async fn update(
        &mut self,
        device_id: &str,
        property_name: &str,
        value: Value,
    ) -> Result<(), String> {
        let (dmx_device, device) = self
            .devices
            .get_mut(device_id)
            .ok_or(format!("Cannot find device '{}'", device_id))?;

        dmx_device
            .update(device, &self.player, property_name, value)
            .await
    }
}
