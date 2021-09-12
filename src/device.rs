/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::device;
use crate::config;
use crate::player::Player;
use serde_json::value::Value;
use std::collections::{BTreeMap, HashMap};
use webthings_gateway_ipc_types::Device;
use webthings_gateway_ipc_types::{Device as DeviceDescription, Property as PropertyDescription};

pub struct DmxDevice {
    pub description: DeviceDescription,
    property_addresses: HashMap<String, u8>,
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

    pub async fn update(
        &mut self,
        device: &mut device::Device,
        player: &Player,
        property_name: &str,
        value: Value,
    ) -> Result<(), String> {
        let address = self.property_addresses.get(property_name).ok_or(format!(
            "Cannot find property address for '{}' in '{}'",
            property_name, self.description.id
        ))?;

        if let Value::Number(value) = value {
            let value = value.as_u64().ok_or(format!(
                "Value {} for {} is not an u64",
                value, property_name
            ))? as u8;

            player
                .set(*address as usize, vec![value])
                .map_err(|err| format!("Could not send DMX value: {}", err))?;

            device
                .set_property_value(property_name, Value::from(value))
                .await
                .map_err(|err| format!("Could not value for property {}: {}", property_name, err))
        } else {
            Err(format!(
                "Value {} for {} is not a number",
                value, property_name
            ))
        }
    }
}
