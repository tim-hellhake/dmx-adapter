/**
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::config;
use crate::descriptions::PropertyDescription;
use crate::player::Player;
use serde_json::value::Value;

pub struct DmxProperty {
    pub description: PropertyDescription,
    address: u8,
}

impl DmxProperty {
    pub fn new(property_config: config::Property) -> Self {
        let description = PropertyDescription {
            property_type: Some(String::from("LevelProperty")),
            name: property_config.id.expect("Properties must have an id"),
            title: property_config.title.clone(),
            description: property_config.title,
            value_type: String::from("integer"),
            unit: None,
            value_enum: None,
            minimum: Some(0 as f32),
            maximum: Some(255 as f32),
            multiple_of: Some(1 as f32),
            read_only: Some(false),
            value: Value::from(0),
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
                    self.description.value = Value::from(value);
                    match player.set(self.address as usize, vec![value as u8]) {
                        Ok(()) => {}
                        Err(err) => println!("Could not send DMX value: {}", err),
                    }
                }
                None => println!("{} is not an u64", self.description.name),
            },
            _ => println!("{} is not a number", self.description.name),
        }
    }
}
