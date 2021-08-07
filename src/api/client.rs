/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use futures::prelude::*;
use futures::stream::SplitSink;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use tungstenite::error::Error as WebSocketError;
use webthings_gateway_ipc_types::{Message as IPCMessage, PluginRegisterRequestMessageData};

use crate::api::plugin::Plugin;

pub struct Client {
    sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
}

impl Client {
    pub fn new(sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>) -> Self {
        Self { sink }
    }

    pub async fn send_message(&mut self, msg: IPCMessage) -> Result<(), String> {
        match serde_json::to_string(&msg) {
            Ok(json) => match self.send(json).await {
                Ok(_) => Ok(()),
                Err(err) => Err(err.to_string()),
            },
            Err(err) => Err(err.to_string()),
        }
    }

    pub async fn send(&mut self, msg: String) -> Result<(), WebSocketError> {
        return self.sink.send(Message::Text(msg.into())).await;
    }

    pub async fn register_plugin(&mut self, plugin_id: &str) -> Result<Plugin, String> {
        let message: IPCMessage = PluginRegisterRequestMessageData {
            plugin_id: plugin_id.to_owned(),
        }
        .into();

        match serde_json::to_string(&message) {
            Ok(json) => match self.send(json).await {
                Ok(_) => Ok(Plugin {
                    plugin_id: plugin_id.to_owned(),
                }),
                Err(err) => Err(err.to_string()),
            },
            Err(err) => Err(err.to_string()),
        }
    }
}
