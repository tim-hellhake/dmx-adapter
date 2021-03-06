/**
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::adapter::Adapter;
use crate::api::client::Client;
use webthings_gateway_ipc_types::{
    AdapterAddedNotificationMessageData, Message, PluginUnloadResponseMessageData,
};

pub struct Plugin {
    pub plugin_id: String,
}

impl Plugin {
    pub async fn create_adapter(
        &self,
        client: &mut Client,
        adapter_id: &str,
        name: &str,
    ) -> Result<Adapter, String> {
        let message: Message = AdapterAddedNotificationMessageData {
            plugin_id: self.plugin_id.clone(),
            adapter_id: adapter_id.to_string(),
            name: name.to_string(),
            package_name: self.plugin_id.clone(),
        }
        .into();

        match serde_json::to_string(&message) {
            Ok(json) => match client.send(json).await {
                Ok(_) => Ok(Adapter {
                    plugin_id: self.plugin_id.clone(),
                    adapter_id: adapter_id.to_string(),
                }),
                Err(err) => Err(err.to_string()),
            },
            Err(err) => Err(err.to_string()),
        }
    }

    pub async fn unload(&self, client: &mut Client) -> Result<(), String> {
        let message: Message = PluginUnloadResponseMessageData {
            plugin_id: self.plugin_id.clone(),
        }
        .into();

        match serde_json::to_string(&message) {
            Ok(json) => match client.send(json).await {
                Ok(_) => Ok(()),
                Err(err) => Err(err.to_string()),
            },
            Err(err) => Err(err.to_string()),
        }
    }
}
