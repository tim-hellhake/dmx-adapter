/**
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::plugin::Plugin;
use tungstenite::client::AutoStream;
use tungstenite::error::Error as WebSocketError;
use tungstenite::{Message, WebSocket};
use url::Url;
use webthings_gateway_ipc_types::{Message as IPCMessage, PluginRegisterRequestMessageData};
use std::str::FromStr;

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

    pub fn read(&mut self) -> Result<IPCMessage, String> {
        match self.socket.read_message() {
            Ok(msg) => match msg.to_text() {
                Ok(json) => {
                    IPCMessage::from_str(json).map_err(|err| format!("{}", err))
                }
                Err(err) => Err(err.to_string()),
            },
            Err(err) => Err(err.to_string()),
        }
    }

    pub fn register_plugin(&mut self, plugin_id: &str) -> Result<Plugin, String> {
        let message: IPCMessage = PluginRegisterRequestMessageData {
            plugin_id: plugin_id.to_string(),
        }.into();

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
