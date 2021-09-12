/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::adapter::DmxAdapter;
use crate::api::adapter::Adapter;
use crate::api::api_error::ApiError;
use crate::api::database::Database;
use crate::api::plugin::{connect, Plugin};
use crate::config::Config;
use std::collections::HashMap;
use std::path::PathBuf;
use webthings_gateway_ipc_types::{
    AdapterUnloadRequest, DeviceSetPropertyCommand, Message as IPCMessage, PluginUnloadRequest,
};

mod adapter;
mod api;
mod config;
mod device;
mod player;

#[tokio::main]
async fn main() {
    run().await.expect("Could not start adapter");
    println!("Exiting adapter");
}

async fn run() -> Result<(), ApiError> {
    let mut plugin = connect("dmx-adapter").await?;
    println!("Plugin registered");
    let mut adapters = HashMap::new();

    let config_path = PathBuf::from(plugin.user_profile.config_dir.clone());
    let database = Database::new(config_path, plugin.plugin_id.clone());
    let conf: Option<Config> = database.load_config()?;

    if let Some(conf) = conf {
        database.save_config(&conf)?;

        for adapter_config in conf.adapters {
            let id = adapter_config.id.clone();

            let title = adapter_config.title.clone();

            println!("Creating adapter '{}' ({})", title, id);

            let mut dmx_adapter = DmxAdapter::new();

            let adapter = plugin.create_adapter(&id, &title).await?;

            if let Err(err) = dmx_adapter.init(&adapter, adapter_config).await {
                plugin
                    .fail(format!("Failed to initialize adapter: {}", err))
                    .await?;
            }

            adapters.insert(id, (dmx_adapter, adapter));
        }
    }

    loop {
        match plugin.read().await {
            None => {}
            Some(result) => match result {
                Ok(message) => match handle_message(&plugin, &mut adapters, message).await {
                    Ok(MessageResult::Continue) => {}
                    Ok(MessageResult::Terminate) => {
                        break;
                    }
                    Err(err) => eprintln!("Failed not handle message: {}", err),
                },
                Err(err) => println!("Could not read message: {}", err),
            },
        }
    }

    Ok(())
}

enum MessageResult {
    Continue,
    Terminate,
}

async fn handle_message(
    plugin: &Plugin,
    adapters: &mut HashMap<String, (DmxAdapter, Adapter)>,
    message: IPCMessage,
) -> Result<MessageResult, String> {
    match message {
        IPCMessage::DeviceSetPropertyCommand(DeviceSetPropertyCommand {
            message_type: _,
            data: message,
        }) => {
            let (dmx_adapter, _) = adapters
                .get_mut(&message.adapter_id)
                .ok_or_else(|| format!("Cannot find adapter '{}'", message.adapter_id))?;

            dmx_adapter
                .update(
                    &message.device_id,
                    &message.property_name,
                    message.property_value.clone(),
                )
                .await
                .map_err(|err| {
                    format!(
                        "Failed to update property {} of {}: {}",
                        message.property_name, message.device_id, err,
                    )
                })?;

            Ok(MessageResult::Continue)
        }
        IPCMessage::AdapterUnloadRequest(AdapterUnloadRequest {
            message_type: _,
            data: message,
        }) => {
            println!(
                "Received request to unload adapter '{}'",
                message.adapter_id
            );

            let (_, adapter) = adapters
                .get_mut(&message.adapter_id)
                .ok_or_else(|| format!("Cannot find adapter '{}'", message.adapter_id))?;

            adapter
                .unload()
                .await
                .map_err(|err| format!("Could not send unload response: {}", err))?;

            Ok(MessageResult::Continue)
        }
        IPCMessage::PluginUnloadRequest(PluginUnloadRequest {
            message_type: _,
            data: message,
        }) => {
            println!("Received request to unload plugin '{}'", message.plugin_id);

            plugin
                .unload()
                .await
                .map_err(|err| format!("Could not send unload response: {}", err))?;

            Ok(MessageResult::Terminate)
        }
        IPCMessage::DeviceSavedNotification(_) => Ok(MessageResult::Continue),
        msg => Err(format!("Unexpected msg: {:?}", msg)),
    }
}
