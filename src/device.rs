/**
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::client::Client;
use crate::api::device::Device;
use crate::config;
use crate::descriptions::DeviceDescription;
use crate::player::Player;
use crate::property::DmxProperty;
use serde_json::value::Value;
use std::collections::HashMap;

pub struct DmxDevice {
    pub description: DeviceDescription,
    properties: HashMap<String, DmxProperty>,
}

impl DmxDevice {
    pub fn new(device_config: config::Device) -> Self {
        let mut properties = HashMap::new();
        let mut property_descriptions = HashMap::new();

        for property_config in device_config.properties {
            let id = property_config
                .id
                .clone()
                .expect("Properties must have an id");

            println!(
                "Creating property '{}' ({}) for '{}'",
                property_config.title, id, device_config.title
            );

            let property = DmxProperty::new(property_config);
            property_descriptions.insert(id.clone(), property.description.clone());
            properties.insert(id, property);
        }

        let description = DeviceDescription {
            context: None,
            types: Some(vec![]),
            id: device_config.id.expect("Devices must have an id"),
            title: String::from("Dmx"),
            description: String::from("A dmx light"),
            properties: property_descriptions,
        };

        return Self {
            description,
            properties,
        };
    }

    pub fn update(
        &mut self,
        client: &mut Client,
        device: &mut Device,
        player: &Player,
        property_name: &str,
        value: &Value,
    ) {
        match self.properties.get_mut(property_name) {
            Some(property) => {
                println!(
                    "Property '{}' in '{}' changed to {}",
                    property.description.title, self.description.title, value
                );
                property.update(player, value);
                match device.update_property(client, &property.description) {
                    Ok(()) => {}
                    Err(err) => println!("Could not update device: {}", err),
                };
            }
            None => println!(
                "Cannot find property '{}' in '{}'",
                property_name, self.description.id
            ),
        }
    }
}
