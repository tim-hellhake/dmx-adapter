/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::client::Client;
use crate::api::device;
use webthings_gateway_ipc_types::{
    AdapterUnloadResponseMessageData, Device, DeviceAddedNotificationMessageData, Message,
};

pub struct Adapter {
    pub plugin_id: String,
    pub adapter_id: String,
}

impl Adapter {
    pub async fn add_device(
        &self,
        client: &mut Client,
        device_description: Device,
    ) -> Result<device::Device, String> {
        let device_id = device_description.id.clone();

        let message: Message = DeviceAddedNotificationMessageData {
            plugin_id: self.plugin_id.clone(),
            adapter_id: self.adapter_id.clone(),
            device: device_description,
        }
        .into();

        match serde_json::to_string(&message) {
            Ok(json) => match client.send(json).await {
                Ok(_) => Ok(device::Device {
                    plugin_id: self.plugin_id.clone(),
                    adapter_id: self.adapter_id.clone(),
                    device_id,
                }),
                Err(err) => Err(err.to_string()),
            },
            Err(err) => Err(err.to_string()),
        }
    }

    pub async fn unload(&self, client: &mut Client) -> Result<(), String> {
        let message: Message = AdapterUnloadResponseMessageData {
            plugin_id: self.plugin_id.clone(),
            adapter_id: self.adapter_id.clone(),
        }
        .into();

        client.send_message(message).await
    }
}
