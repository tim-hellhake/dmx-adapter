/**
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::message::{
    AdapterUnloadRequest, DeviceSetPropertyCommand, IncomingMessage, IncomingPayloadMessage,
    PayloadMessage, PluginRegisterRequest, PluginRegisterResponse, PluginUnloadRequest,
    TypeMessage,
};
use crate::api::plugin::Plugin;
use serde::Deserialize;
use serde_json::error::Error as SerdeJsonError;
use tungstenite::client::AutoStream;
use tungstenite::error::Error as WebSocketError;
use tungstenite::{Message, WebSocket};
use url::Url;

pub struct Client {
    socket: WebSocket<AutoStream>,
}

impl Client {
    pub fn connect(url: Url) -> Result<Self, String> {
        match tungstenite::connect(url) {
            Ok((socket, _)) => Ok(Self { socket }),
            Err(err) => Err(err.to_string()),
        }
    }

    pub fn send(&mut self, msg: String) -> Result<(), WebSocketError> {
        return self.socket.write_message(Message::Text(msg.into()));
    }

    pub fn read(&mut self) -> Result<IncomingMessage, String> {
        match self.socket.read_message() {
            Ok(msg) => match msg.to_text() {
                Ok(json) => {
                    let type_result: Result<TypeMessage, SerdeJsonError> =
                        serde_json::from_str(json);

                    return match type_result {
                        Ok(message) => {
                            return match message.message_type {
                                1 => {
                                    return Client::parse_message(
                                        json,
                                        |x: PluginRegisterResponse| {
                                            IncomingMessage::PluginRegister(x)
                                        },
                                    )
                                }
                                8198 => {
                                    return Client::parse_message(
                                        json,
                                        |x: DeviceSetPropertyCommand| {
                                            IncomingMessage::DeviceSetProperty(x)
                                        },
                                    )
                                }
                                2 => {
                                    return Client::parse_message(json, |x: PluginUnloadRequest| {
                                        IncomingMessage::PluginUnload(x)
                                    })
                                }
                                4097 => {
                                    return Client::parse_message(
                                        json,
                                        |x: AdapterUnloadRequest| IncomingMessage::AdapterUnload(x),
                                    )
                                }
                                _ => Err(format!(
                                    "Received unknown message type {}",
                                    message.message_type
                                )),
                            };
                        }
                        Err(err) => Err(err.to_string()),
                    };
                }
                Err(err) => Err(err.to_string()),
            },
            Err(err) => Err(err.to_string()),
        }
    }

    pub fn parse_message<'a, T, F>(json: &'a str, f: F) -> Result<IncomingMessage, String>
    where
        F: Fn(T) -> IncomingMessage,
        T: Deserialize<'a>,
    {
        let p: Result<IncomingPayloadMessage<T>, SerdeJsonError> = serde_json::from_str(json);

        return match p {
            Ok(message) => Ok(f(message.data)),
            Err(err) => Err(err.to_string()),
        };
    }

    pub fn register_plugin(&mut self, plugin_id: &str) -> Result<Plugin, String> {
        let message = PayloadMessage {
            message_type: 0,
            data: &PluginRegisterRequest {
                plugin_id: plugin_id.to_string(),
            },
        };

        match serde_json::to_string(&message) {
            Ok(json) => match self.send(json) {
                Ok(_) => Ok(Plugin {
                    plugin_id: plugin_id.to_string(),
                }),
                Err(err) => Err(err.to_string()),
            },
            Err(err) => Err(err.to_string()),
        }
    }
}
