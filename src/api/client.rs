/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use futures::prelude::*;
use futures::stream::SplitSink;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
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

    pub async fn send_message(&mut self, msg: &IPCMessage) -> Result<(), String> {
        let json = serde_json::to_string(msg)
            .map_err(|err| format!("Could not serialize value: {}", err))?;

        self.send(json)
            .await
            .map_err(|err| format!("Could not send json: {:?}", err))
    }

    pub async fn send(&mut self, msg: String) -> Result<(), WebSocketError> {
        self.sink.send(Message::Text(msg)).await
    }

    pub async fn register_plugin(mut self, plugin_id: &str) -> Result<Plugin, String> {
        let message: IPCMessage = PluginRegisterRequestMessageData {
            plugin_id: plugin_id.to_owned(),
        }
        .into();

        self.send_message(&message).await?;

        Ok(Plugin {
            plugin_id: plugin_id.to_owned(),
            client: Arc::new(Mutex::new(self)),
        })
    }
}
