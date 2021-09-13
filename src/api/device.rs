/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::api_error::ApiError;
use crate::api::client::Client;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use webthings_gateway_ipc_types::{
    Device as DeviceDescription, DevicePropertyChangedNotificationMessageData, Message,
};

#[async_trait(?Send)]
pub trait DeviceHandler {
    async fn on_property_updated(
        &self,
        device: &mut Device,
        name: &str,
        value: Value,
    ) -> Result<(), String>;
}

pub struct Device {
    client: Arc<Mutex<Client>>,
    pub plugin_id: String,
    pub adapter_id: String,
    pub description: DeviceDescription,
    device_handler: Option<Arc<Mutex<dyn DeviceHandler + Send>>>,
}

impl Device {
    pub fn new(
        client: Arc<Mutex<Client>>,
        plugin_id: String,
        adapter_id: String,
        description: DeviceDescription,
    ) -> Self {
        Device {
            client,
            plugin_id,
            adapter_id,
            description,
            device_handler: None,
        }
    }

    pub fn set_device_handler(&mut self, device_handler: Arc<Mutex<dyn DeviceHandler + Send>>) {
        self.device_handler = Some(device_handler);
    }

    pub async fn set_property_value(&mut self, name: &str, value: Value) -> Result<(), ApiError> {
        let device_id = self.description.id.clone();
        let properties = &mut self.description.properties;

        match properties {
            Some(properties) => {
                let mut property =
                    properties
                        .get_mut(name)
                        .ok_or_else(|| ApiError::PropertyNotFound {
                            device_id,
                            property_name: name.to_owned(),
                        })?;

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
            None => Err(ApiError::PropertyNotFound {
                device_id,
                property_name: name.to_owned(),
            }),
        }
    }

    pub async fn on_property_updated(&mut self, name: &str, value: Value) -> Result<(), String> {
        if let Some(device_handler) = self.device_handler.clone() {
            device_handler
                .lock()
                .await
                .on_property_updated(self, name, value)
                .await?;
        }

        Ok(())
    }
}
