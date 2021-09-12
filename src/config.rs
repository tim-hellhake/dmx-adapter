/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub adapters: Vec<Adapter>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Adapter {
    #[serde(default = "uuid")]
    pub id: String,
    pub title: String,
    pub serial_port: String,
    pub devices: Vec<Device>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    #[serde(default = "uuid")]
    pub id: String,
    pub title: String,
    pub properties: Vec<Property>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    #[serde(default = "uuid")]
    pub id: String,
    pub title: String,
    pub address: u8,
}

fn uuid() -> String {
    Uuid::new_v4().to_string()
}
