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
use futures::prelude::*;
use futures::stream::SplitStream;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use url::Url;
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
    match connect_async(Url::parse("ws://localhost:9500").expect("Could not parse url")).await {
        Ok((socket, _)) => {
            let (sink, mut stream) = socket.split();
            let client = Client::new(sink);

            let plugin = client
                .register_plugin("dmx-adapter")
                .await
                .expect("Could not register plugin");

            let mut adapters = HashMap::new();

            loop {
                match read(&mut stream).await {
                    None => {}
                    Some(result) => match result {
                        Ok(message) => {
                            match handle_message(&plugin, &mut adapters, message).await {
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
        }
        Err(err) => eprintln!("Could not connect to gateway: {:?}", err),
    }

    println!("Exiting adapter");
}

async fn read(
    stream: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> Option<Result<IPCMessage, String>> {
    stream.next().await.map(|result| match result {
        Ok(msg) => match msg.to_text() {
            Ok(json) => IPCMessage::from_str(json).map_err(|err| format!("{}", err)),
            Err(err) => Err(err.to_string()),
        },
        Err(err) => Err(err.to_string()),
    })
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
                    .create_adapter(&id, &title)
                    .await
                    .expect("Could not create adapter");

                dmx_adapter
                    .init(&adapter, adapter_config)
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
                if let Err(err) = dmx_adapter
                    .update(
                        &message.device_id,
                        &message.property_name,
                        message.property_value,
                    )
                    .await
                {
                    eprintln!(
                        "Failed to update property {} of {}: {}",
                        message.property_name, message.device_id, err
                    );
                }

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
                Some((_, adapter)) => match adapter.unload().await {
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

            match plugin.unload().await {
                Ok(_) => Ok(MessageResult::Terminate),
                Err(msg) => Err(format!("Could not send unload response: {}", msg)),
            }
        }
        IPCMessage::DeviceSavedNotification(_) => Ok(MessageResult::Continue),
        msg => Err(format!("Unexpected msg: {:?}", msg)),
    }
}
