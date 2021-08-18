/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::adapter::Adapter;
use crate::api::client::Client;
use std::sync::Arc;
use tokio::sync::Mutex;
use webthings_gateway_ipc_types::{
    AdapterAddedNotificationMessageData, Message, PluginUnloadResponseMessageData,
};

pub struct Plugin {
    pub plugin_id: String,
    pub client: Arc<Mutex<Client>>,
}

impl Plugin {
    pub async fn create_adapter(&self, adapter_id: &str, name: &str) -> Result<Adapter, String> {
        let message: Message = AdapterAddedNotificationMessageData {
            plugin_id: self.plugin_id.clone(),
            adapter_id: adapter_id.to_owned(),
            name: name.to_owned(),
            package_name: self.plugin_id.clone(),
        }
        .into();

        self.client.lock().await.send_message(&message).await?;

        Ok(Adapter {
            client: self.client.clone(),
            plugin_id: self.plugin_id.clone(),
            adapter_id: adapter_id.to_owned(),
        })
    }

    pub async fn unload(&self) -> Result<(), String> {
        let message: Message = PluginUnloadResponseMessageData {
            plugin_id: self.plugin_id.clone(),
        }
        .into();

        self.client.lock().await.send_message(&message).await
    }
}
