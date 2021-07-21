/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use std::str::FromStr;

use futures::prelude::*;
use futures::stream::{SplitSink, SplitStream};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use tungstenite::error::Error as WebSocketError;
use url::Url;
use webthings_gateway_ipc_types::{Message as IPCMessage, PluginRegisterRequestMessageData};

use crate::api::plugin::Plugin;

pub struct Client {
    sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl Client {
    pub async fn connect(url: Url) -> Result<Self, String> {
        match connect_async(url).await {
            Ok((socket, _)) => {
                let (sink, stream) = socket.split();
                Ok(Self { sink, stream })
            }
            Err(err) => Err(err.to_string()),
        }
    }

    pub async fn send(&mut self, msg: String) -> Result<(), WebSocketError> {
        return self.sink.send(Message::Text(msg.into())).await;
    }

    pub async fn read(&mut self) -> Option<Result<IPCMessage, String>> {
        self.stream
            .next()
            .await
            .map(|result| match result {
                Ok(msg) => match msg.to_text() {
                    Ok(json) => IPCMessage::from_str(json).map_err(|err| format!("{}", err)),
                    Err(err) => Err(err.to_string()),
                },
                Err(err) => Err(err.to_string()),
            })
            .into()
    }

    pub async fn register_plugin(&mut self, plugin_id: &str) -> Result<Plugin, String> {
        let message: IPCMessage = PluginRegisterRequestMessageData {
            plugin_id: plugin_id.to_string(),
        }
        .into();

        match serde_json::to_string(&message) {
            Ok(json) => match self.send(json).await {
                Ok(_) => Ok(Plugin {
                    plugin_id: plugin_id.to_string(),
                }),
                Err(err) => Err(err.to_string()),
            },
            Err(err) => Err(err.to_string()),
        }
    }
}
