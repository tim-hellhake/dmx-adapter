/**
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::descriptions::{DeviceDescription, PropertyDescription};
use serde::{Deserialize, Serialize};
use serde_json::value::Value;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PayloadMessage<'a, T> {
    pub message_type: u32,
    pub data: &'a T,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PluginRegisterRequest {
    pub plugin_id: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AdapterAddedNotification {
    pub plugin_id: String,
    pub adapter_id: String,
    pub name: String,
    pub package_name: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeviceAddedNotification<'a> {
    pub plugin_id: String,
    pub adapter_id: String,
    pub device: &'a DeviceDescription,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DevicePropertyChangedNotification<'a> {
    pub plugin_id: String,
    pub adapter_id: String,
    pub device_id: String,
    pub property: &'a PropertyDescription,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PluginUnloadResponse {
    pub plugin_id: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AdapterUnloadResponse {
    pub plugin_id: String,
    pub adapter_id: String,
}

#[derive(Debug)]
pub enum IncomingMessage {
    PluginRegister(PluginRegisterResponse),
    DeviceSetProperty(DeviceSetPropertyCommand),
    PluginUnload(PluginUnloadRequest),
    AdapterUnload(AdapterUnloadRequest),
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PluginRegisterResponse {
    pub plugin_id: String,
    pub gateway_version: String,
    pub user_profile: UserProfile,
    pub preferences: Preferences,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub addons_dir: String,
    pub base_dir: String,
    pub config_dir: String,
    pub data_dir: String,
    pub media_dir: String,
    pub log_dir: String,
    pub gateway_dir: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub language: String,
    pub units: Units,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Units {
    pub temperature: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSetPropertyCommand {
    pub plugin_id: String,
    pub adapter_id: String,
    pub device_id: String,
    pub property_name: String,
    pub property_value: Value,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PluginUnloadRequest {
    pub plugin_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AdapterUnloadRequest {
    pub plugin_id: String,
    pub adapter_id: String,
}
