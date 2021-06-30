/**
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::client::Client;
use webthings_gateway_ipc_types::{DevicePropertyChangedNotificationMessageData, Property, Message};

pub struct Device {
    pub plugin_id: String,
    pub adapter_id: String,
    pub device_id: String,
}

impl Device {
    pub fn update_property(
        &self,
        client: &mut Client,
        property_description: Property,
    ) -> Result<(), String> {
        let message: Message = DevicePropertyChangedNotificationMessageData {
            plugin_id: self.plugin_id.clone(),
            adapter_id: self.adapter_id.clone(),
            device_id: self.device_id.clone(),
            property: property_description,
        }.into();

        match serde_json::to_string(&message) {
            Ok(json) => match client.send(json) {
                Ok(_) => Ok(()),
                Err(err) => Err(err.to_string()),
            },
            Err(err) => Err(err.to_string()),
        }
    }
}
