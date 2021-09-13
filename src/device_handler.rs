/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::device::{Device, DeviceHandler};
use crate::device::DmxDevice;
use crate::player::Player;
use async_trait::async_trait;
use serde_json::value::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct DmxDeviceHandler {
    dmx_device: DmxDevice,
    player: Arc<Mutex<Player>>,
}

impl DmxDeviceHandler {
    pub fn new(dmx_device: DmxDevice, player: Arc<Mutex<Player>>) -> Self {
        DmxDeviceHandler { dmx_device, player }
    }
}

#[async_trait(?Send)]
impl DeviceHandler for DmxDeviceHandler {
    async fn on_property_updated(
        &self,
        device: &mut Device,
        name: &str,
        value: Value,
    ) -> Result<(), String> {
        let address = self.dmx_device.property_addresses.get(name).ok_or(format!(
            "Cannot find property address for '{}' in '{}'",
            name, self.dmx_device.description.id
        ))?;

        if let Value::Number(value) = value {
            let value = value
                .as_u64()
                .ok_or(format!("Value {} for {} is not an u64", value, name))?
                as u8;

            self.player
                .lock()
                .await
                .set(*address as usize, vec![value])
                .map_err(|err| format!("Could not send DMX value: {}", err))?;

            device
                .set_property_value(name, Value::from(value))
                .await
                .map_err(|err| format!("Could not value for property {}: {}", name, err))
        } else {
            Err(format!("Value {} for {} is not a number", value, name))
        }
    }
}
