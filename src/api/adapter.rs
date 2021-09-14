/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::api_error::ApiError;
use crate::api::client::Client;
use crate::api::device;
use crate::api::device::Device;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use webthings_gateway_ipc_types::{
    AdapterUnloadResponseMessageData, Device as DeviceDescription,
    DeviceAddedNotificationMessageData, Message,
};

pub struct AdapterHandle {
    client: Arc<Mutex<Client>>,
    pub plugin_id: String,
    pub adapter_id: String,
    devices: HashMap<String, Arc<Mutex<dyn Device>>>,
}

impl AdapterHandle {
    pub fn new(client: Arc<Mutex<Client>>, plugin_id: String, adapter_id: String) -> Self {
        Self {
            client,
            plugin_id,
            adapter_id,
            devices: HashMap::new(),
        }
    }

    pub async fn add_device<T, F>(
        &mut self,
        device_description: DeviceDescription,
        constructor: F,
    ) -> Result<Arc<Mutex<T>>, ApiError>
    where
        T: Device + 'static,
        F: FnOnce(device::DeviceHandle) -> T,
    {
        let message: Message = DeviceAddedNotificationMessageData {
            plugin_id: self.plugin_id.clone(),
            adapter_id: self.adapter_id.clone(),
            device: device_description.clone(),
        }
        .into();

        self.client.lock().await.send_message(&message).await?;

        let id = device_description.id.clone();

        let device = device::DeviceHandle::new(
            self.client.clone(),
            self.plugin_id.clone(),
            self.adapter_id.clone(),
            device_description,
        );

        let device_handler = Arc::new(Mutex::new(constructor(device)));

        self.devices.insert(id, device_handler.clone());

        Ok(device_handler)
    }

    pub fn get_device(&self, id: &str) -> Option<Arc<Mutex<dyn Device>>> {
        self.devices.get(id).cloned()
    }

    pub async fn unload(&self) -> Result<(), ApiError> {
        let message: Message = AdapterUnloadResponseMessageData {
            plugin_id: self.plugin_id.clone(),
            adapter_id: self.adapter_id.clone(),
        }
        .into();

        self.client.lock().await.send_message(&message).await
    }
}
