/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::adapter::DmxAdapter;
use crate::api::client::Client;
use crate::api::database::Database;
use crate::config::Config;
use std::collections::HashMap;
use std::path::PathBuf;
use url::Url;
use webthings_gateway_ipc_types::{
    AdapterUnloadRequest, DeviceSetPropertyCommand, Message as IPCMessage, PluginUnloadRequest,
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
                Ok(message) => match message {
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
                        }
                        None => println!("Cannot find adapter '{}'", message.adapter_id),
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
                            Some((_, adapter)) => {
                                match adapter.unload(&mut client).await {
                                    Ok(_) => {}
                                    Err(msg) => println!("Could not send unload response: {}", msg),
                                };
                            }
                            None => println!("Cannot find adapter '{}'", message.adapter_id),
                        }
                    }
                    IPCMessage::PluginUnloadRequest(PluginUnloadRequest {
                        message_type: _,
                        data: message,
                    }) => {
                        println!("Received request to unload plugin '{}'", message.plugin_id);

                        match plugin.unload(&mut client).await {
                            Ok(_) => {}
                            Err(msg) => println!("Could not send unload response: {}", msg),
                        };

                        break;
                    }
                    IPCMessage::DeviceSavedNotification(_) => {}
                    msg => {
                        eprintln!("Unexpected msg: {:?}", msg);
                    }
                },
                Err(err) => println!("Could not read message: {}", err),
            },
        }
    }

    println!("Exiting adapter");
}
