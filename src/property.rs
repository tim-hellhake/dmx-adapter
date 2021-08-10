/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::config;
use crate::player::Player;
use serde_json::value::Value;
use webthings_gateway_ipc_types::Property;

pub struct DmxProperty {
    pub description: Property,
    address: u8,
}

impl DmxProperty {
    pub fn new(property_config: config::Property) -> Self {
        let description = Property {
            at_type: Some(String::from("LevelProperty")),
            name: Some(property_config.id.expect("Properties must have an id")),
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

        DmxProperty {
            description,
            address: property_config.address,
        }
    }

    pub fn update(&mut self, player: &Player, value: &Value) {
        match value {
            Value::Number(value) => match value.as_u64() {
                Some(value) => {
                    self.description.value = Some(Value::from(value));
                    match player.set(self.address as usize, vec![value as u8]) {
                        Ok(()) => {}
                        Err(err) => println!("Could not send DMX value: {}", err),
                    }
                }
                None => println!("{:?} is not an u64", self.description.name),
            },
            _ => println!("{:?} is not a number", self.description.name),
        }
    }
}
