/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use futures::prelude::*;
use futures::stream::SplitSink;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use webthings_gateway_ipc_types::Message as IPCMessage;

use crate::api::api_error::ApiError;

pub struct Client {
    sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
}

impl Client {
    pub fn new(sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>) -> Self {
        Self { sink }
    }

    pub async fn send_message(&mut self, msg: &IPCMessage) -> Result<(), ApiError> {
        let json = serde_json::to_string(msg).map_err(ApiError::Serialization)?;

        self.send(json).await
    }

    pub async fn send(&mut self, msg: String) -> Result<(), ApiError> {
        self.sink
            .send(Message::Text(msg))
            .await
            .map_err(ApiError::Send)
    }
}
