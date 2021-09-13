/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::config;
use serde_json::value::Value;
use std::collections::{BTreeMap, HashMap};
use webthings_gateway_ipc_types::Device;
use webthings_gateway_ipc_types::{Device as DeviceDescription, Property as PropertyDescription};

pub struct DmxDevice {
    pub description: DeviceDescription,
    pub property_addresses: HashMap<String, u8>,
}

impl DmxDevice {
    pub fn new(device_config: config::Device) -> Self {
        let mut property_addresses = HashMap::new();
        let mut property_descriptions = BTreeMap::new();

        for property_config in device_config.properties {
            let id = property_config.id.clone();

            println!(
                "Creating property '{}' ({}) for '{}'",
                property_config.title, id, device_config.title
            );

            let description = PropertyDescription {
                at_type: Some(String::from("LevelProperty")),
                name: Some(property_config.id),
                title: Some(property_config.title.clone()),
                description: Some(property_config.title),
                type_: String::from("integer"),
                unit: None,
                enum_: None,
                links: None,
                minimum: Some(0 as f64),
                maximum: Some(255_f64),
                multiple_of: Some(1_f64),
                read_only: Some(false),
                value: Some(Value::from(0)),
                visible: Some(true),
            };

            property_descriptions.insert(id.clone(), description);
            property_addresses.insert(id, property_config.address);
        }

        let description = Device {
            at_context: None,
            at_type: Some(vec![]),
            id: device_config.id,
            title: Some(String::from("Dmx")),
            description: Some(String::from("A dmx light")),
            properties: Some(property_descriptions),
            actions: None,
            events: None,
            links: None,
            base_href: None,
            pin: None,
            credentials_required: None,
        };

        Self {
            description,
            property_addresses,
        }
    }
}
