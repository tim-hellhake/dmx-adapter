/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::client::Client;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use webthings_gateway_ipc_types::{
    Device as DeviceDescription, DevicePropertyChangedNotificationMessageData, Message,
};

pub struct Device {
    pub client: Arc<Mutex<Client>>,
    pub plugin_id: String,
    pub adapter_id: String,
    pub description: DeviceDescription,
}

impl Device {
    pub async fn set_property_value(&mut self, name: &str, value: Value) -> Result<(), String> {
        let properties = &mut self.description.properties;

        match properties {
            Some(properties) => {
                let id = self.description.id.clone();

                let mut property = properties
                    .get_mut(name)
                    .ok_or_else(|| format!("No property called {} found in {}", name, id))?;

                property.value = Some(value);

                let message: Message = DevicePropertyChangedNotificationMessageData {
                    plugin_id: self.plugin_id.clone(),
                    adapter_id: self.adapter_id.clone(),
                    device_id: self.description.id.clone(),
                    property: property.clone(),
                }
                .into();

                self.client.lock().await.send_message(&message).await
            }
            None => Err(format!("Device {} has no properties", self.description.id)),
        }
    }
}
