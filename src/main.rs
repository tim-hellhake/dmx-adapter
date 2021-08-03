/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::adapter::DmxAdapter;
use crate::api::adapter::Adapter;
use crate::api::client::Client;
use crate::api::database::Database;
use crate::api::plugin::Plugin;
use crate::config::Config;
use std::collections::HashMap;
use std::path::PathBuf;
use url::Url;
use webthings_gateway_ipc_types::{
    AdapterUnloadRequest, DeviceSetPropertyCommand, Message as IPCMessage, Message,
    PluginUnloadRequest,
};

mod adapter;
mod api;
mod config;
mod device;
mod player;
mod property;

#[tokio::main]
async fn main() {
    let mut client =
        Client::connect(Url::parse("ws://localhost:9500").expect("Could not parse url"))
            .await
            .expect("Could not connect to gateway");

    let plugin = client
        .register_plugin("dmx-adapter")
        .await
        .expect("Could not register plugin");

    let mut adapters = HashMap::new();

    loop {
        match client.read().await {
            None => {}
            Some(result) => match result {
                Ok(message) => {
                    match handle_message(&mut client, &plugin, &mut adapters, message).await {
                        Ok(MessageResult::Continue) => {}
                        Ok(MessageResult::Terminate) => {
                            break;
                        }
                        Err(err) => eprintln!("Failed not handle message: {}", err),
                    }
                }
                Err(err) => println!("Could not read message: {}", err),
            },
        }
    }

    println!("Exiting adapter");
}

enum MessageResult {
    Continue,
    Terminate,
}

async fn handle_message(
    mut client: &mut Client,
    plugin: &Plugin,
    adapters: &mut HashMap<String, (DmxAdapter, Adapter)>,
    message: Message,
) -> Result<MessageResult, String> {
    match message {
        IPCMessage::PluginRegisterResponse(msg) => {
            let config_path = PathBuf::from(msg.data.user_profile.config_dir);
            let database = Database::new(config_path);
            let mut conf: Config = database.load_config();
            conf.generate_ids();
            database.save_config(&conf);

            println!("Plugin registered");

            for adapter_config in conf.adapters {
                let id = adapter_config
                    .id
                    .as_ref()
                    .expect("adapters must have an id")
                    .clone();

                let title = adapter_config.title.clone();

                println!("Creating adapter '{}' ({})", title, id);

                let mut dmx_adapter = DmxAdapter::new();

                let adapter = plugin
                    .create_adapter(&mut client, &id, &title)
                    .await
                    .expect("Could not create adapter");

                dmx_adapter
                    .init(&mut client, &adapter, adapter_config)
                    .await
                    .expect("Could not initialize adapter");

                adapters.insert(id, (dmx_adapter, adapter));
            }

            Ok(MessageResult::Continue)
        }
        IPCMessage::DeviceSetPropertyCommand(DeviceSetPropertyCommand {
            message_type: _,
            data: message,
        }) => match adapters.get_mut(&message.adapter_id) {
            Some((dmx_adapter, _)) => {
                dmx_adapter
                    .update(
                        &mut client,
                        &message.device_id,
                        &message.property_name,
                        &message.property_value,
                    )
                    .await;

                Ok(MessageResult::Continue)
            }
            None => Err(format!("Cannot find adapter '{}'", message.adapter_id)),
        },
        IPCMessage::AdapterUnloadRequest(AdapterUnloadRequest {
            message_type: _,
            data: message,
        }) => {
            println!(
                "Received request to unload adapter '{}'",
                message.adapter_id
            );

            match adapters.get_mut(&message.adapter_id) {
                Some((_, adapter)) => match adapter.unload(&mut client).await {
                    Ok(_) => Ok(MessageResult::Continue),
                    Err(msg) => Err(format!("Could not send unload response: {}", msg)),
                },
                None => Err(format!("Cannot find adapter '{}'", message.adapter_id)),
            }
        }
        IPCMessage::PluginUnloadRequest(PluginUnloadRequest {
            message_type: _,
            data: message,
        }) => {
            println!("Received request to unload plugin '{}'", message.plugin_id);

            match plugin.unload(&mut client).await {
                Ok(_) => Ok(MessageResult::Terminate),
                Err(msg) => Err(format!("Could not send unload response: {}", msg)),
            }
        }
        IPCMessage::DeviceSavedNotification(_) => Ok(MessageResult::Continue),
        msg => Err(format!("Unexpected msg: {:?}", msg)),
    }
}
