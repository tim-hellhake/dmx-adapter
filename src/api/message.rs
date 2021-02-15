use serde::{Deserialize, Serialize};
use webthings_gateway_ipc_types::{
    AdapterUnloadRequestData, DeviceSetPropertyCommandData, PluginRegisterResponseData,
    PluginUnloadRequestData,
};

/**
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PayloadMessage<'a, T> {
    pub message_type: u32,
    pub data: &'a T,
}

#[derive(Debug)]
pub enum IncomingMessage {
    PluginRegister(PluginRegisterResponseData),
    DeviceSetProperty(DeviceSetPropertyCommandData),
    PluginUnload(PluginUnloadRequestData),
    AdapterUnload(AdapterUnloadRequestData),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TypeMessage {
    pub message_type: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IncomingPayloadMessage<T> {
    pub message_type: u32,
    pub data: T,
}
