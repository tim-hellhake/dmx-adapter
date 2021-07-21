/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::database;
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
    pub id: Option<String>,
    pub title: String,
    pub serial_port: String,
    pub devices: Vec<Device>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: Option<String>,
    pub title: String,
    pub properties: Vec<Property>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    pub id: Option<String>,
    pub title: String,
    pub address: u8,
}

pub fn load() -> Config {
    let json = database::load_string();
    let config: Config = serde_json::from_str(json.as_str()).expect("Could not parse config");
    return config;
}

pub fn generate_ids(config: &mut Config) {
    for adapter in &mut config.adapters {
        if adapter.id.is_none() {
            adapter.id = Some(String::from(Uuid::new_v4().to_string()));
        }

        for device in &mut adapter.devices {
            if device.id.is_none() {
                device.id = Some(String::from(Uuid::new_v4().to_string()));
            }

            for property in &mut device.properties {
                if property.id.is_none() {
                    property.id = Some(String::from(Uuid::new_v4().to_string()));
                }
            }
        }
    }
}

pub fn save(config: &Config) {
    let json = serde_json::to_string(config).expect("Could not serialize config");
    database::save_string(json);
}
