/**
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::adapter::DmxAdapter;
use crate::api::client::Client;
use crate::api::message::IncomingMessage;
use std::collections::HashMap;
use url::Url;

mod adapter;
mod api;
mod config;
mod descriptions;
mod device;
mod player;
mod property;

fn main() {
    let mut client =
        Client::connect(Url::parse("ws://localhost:9500").expect("Could not parse url"))
            .expect("Could not connect to gateway");

    let plugin = client
        .register_plugin("dmx-adapter")
        .expect("Could not register plugin");

    let mut adapters = HashMap::new();

    loop {
        match client.read() {
            Ok(message) => match message {
                IncomingMessage::PluginRegister(_) => {
                    let mut conf = config::load();
                    config::generate_ids(&mut conf);
                    config::save(&conf);

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
                            .expect("Could not create adapter");

                        dmx_adapter
                            .init(&mut client, &adapter, adapter_config)
                            .expect("Could not initialize adapter");

                        adapters.insert(id, (dmx_adapter, adapter));
                    }
                }
                IncomingMessage::DeviceSetProperty(message) => {
                    match adapters.get_mut(&message.adapter_id) {
                        Some((dmx_adapter, _)) => {
                            dmx_adapter.update(
                                &mut client,
                                &message.device_id,
                                &message.property_name,
                                &message.property_value,
                            );
                        }
                        None => println!("Cannot find adapter '{}'", message.adapter_id),
                    }
                }
                IncomingMessage::AdapterUnload(message) => {
                    println!(
                        "Received request to unload adapter '{}'",
                        message.adapter_id
                    );

                    match adapters.get_mut(&message.adapter_id) {
                        Some((_, adapter)) => {
                            match adapter.unload(&mut client) {
                                Ok(_) => {}
                                Err(msg) => println!("Could not send unload response: {}", msg),
                            };
                        }
                        None => println!("Cannot find adapter '{}'", message.adapter_id),
                    }
                }
                IncomingMessage::PluginUnload(message) => {
                    println!("Received request to unload plugin '{}'", message.plugin_id);

                    match plugin.unload(&mut client) {
                        Ok(_) => {}
                        Err(msg) => println!("Could not send unload response: {}", msg),
                    };

                    break;
                }
            },
            Err(err) => println!("Could not read message: {}", err),
        }
    }

    println!("Exiting adapter");
}
