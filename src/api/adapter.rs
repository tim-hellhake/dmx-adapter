/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::client::Client;
use crate::api::device;
use std::sync::Arc;
use tokio::sync::Mutex;
use webthings_gateway_ipc_types::{
    AdapterUnloadResponseMessageData, Device, DeviceAddedNotificationMessageData, Message,
};

pub struct Adapter {
    pub client: Arc<Mutex<Client>>,
    pub plugin_id: String,
    pub adapter_id: String,
}

impl Adapter {
    pub async fn add_device(&self, device_description: Device) -> Result<device::Device, String> {
        let message: Message = DeviceAddedNotificationMessageData {
            plugin_id: self.plugin_id.clone(),
            adapter_id: self.adapter_id.clone(),
            device: device_description.clone(),
        }
        .into();

        match self.client.lock().await.send_message(message).await {
            Ok(_) => Ok(device::Device {
                client: self.client.clone(),
                plugin_id: self.plugin_id.clone(),
                adapter_id: self.adapter_id.clone(),
                description: device_description,
            }),
            Err(err) => Err(err),
        }
    }

    pub async fn unload(&self) -> Result<(), String> {
        let message: Message = AdapterUnloadResponseMessageData {
            plugin_id: self.plugin_id.clone(),
            adapter_id: self.adapter_id.clone(),
        }
        .into();

        self.client.lock().await.send_message(message).await
    }
}
