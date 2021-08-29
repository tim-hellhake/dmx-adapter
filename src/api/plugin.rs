/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */

use crate::api::adapter::Adapter;
use crate::api::api_error::ApiError;
use crate::api::client::Client;
use futures::prelude::*;
use futures::stream::SplitStream;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use url::Url;
use webthings_gateway_ipc_types::{
    AdapterAddedNotificationMessageData, Message, PluginRegisterRequestMessageData,
    PluginUnloadResponseMessageData, Preferences, UserProfile,
};
use webthings_gateway_ipc_types::{Message as IPCMessage, PluginRegisterResponseMessageData};

pub async fn connect(plugin_id: &str) -> Result<Plugin, ApiError> {
    let url = Url::parse("ws://localhost:9500").expect("Could not parse url");

    let (socket, _) = connect_async(url).await.map_err(ApiError::Connect)?;

    let (sink, mut stream) = socket.split();
    let mut client = Client::new(sink);

    let message: IPCMessage = PluginRegisterRequestMessageData {
        plugin_id: plugin_id.to_owned(),
    }
    .into();

    client.send_message(&message).await?;

    let PluginRegisterResponseMessageData {
        gateway_version: _,
        plugin_id: _,
        preferences,
        user_profile,
    } = loop {
        match read(&mut stream).await {
            None => {}
            Some(result) => match result {
                Ok(IPCMessage::PluginRegisterResponse(msg)) => {
                    break msg.data;
                }
                Ok(msg) => {
                    eprintln!("Received unexpected message {:?}", msg);
                }
                Err(err) => println!("Could not read message: {}", err),
            },
        }
    };

    Ok(Plugin {
        plugin_id: plugin_id.to_owned(),
        preferences,
        user_profile,
        client: Arc::new(Mutex::new(client)),
        stream,
    })
}

async fn read(
    stream: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> Option<Result<IPCMessage, String>> {
    stream.next().await.map(|result| match result {
        Ok(msg) => {
            let json = msg
                .to_text()
                .map_err(|err| format!("Could not get text message: {:?}", err))?;

            IPCMessage::from_str(json).map_err(|err| format!("Could not parse message: {:?}", err))
        }
        Err(err) => Err(err.to_string()),
    })
}

pub struct Plugin {
    pub plugin_id: String,
    pub preferences: Preferences,
    pub user_profile: UserProfile,
    client: Arc<Mutex<Client>>,
    stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl Plugin {
    pub async fn create_adapter(&self, adapter_id: &str, name: &str) -> Result<Adapter, ApiError> {
        let message: Message = AdapterAddedNotificationMessageData {
            plugin_id: self.plugin_id.clone(),
            adapter_id: adapter_id.to_owned(),
            name: name.to_owned(),
            package_name: self.plugin_id.clone(),
        }
        .into();

        self.client.lock().await.send_message(&message).await?;

        Ok(Adapter {
            client: self.client.clone(),
            plugin_id: self.plugin_id.clone(),
            adapter_id: adapter_id.to_owned(),
        })
    }

    pub async fn unload(&self) -> Result<(), ApiError> {
        let message: Message = PluginUnloadResponseMessageData {
            plugin_id: self.plugin_id.clone(),
        }
        .into();

        self.client.lock().await.send_message(&message).await
    }

    pub async fn read(&mut self) -> Option<Result<IPCMessage, String>> {
        read(&mut self.stream).await
    }
}
