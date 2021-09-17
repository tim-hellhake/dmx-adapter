/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::property::{Property, PropertyHandle};
use crate::config::Property as PropertyConfig;
use crate::player::Player;
use async_trait::async_trait;
use serde_json::value::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use webthings_gateway_ipc_types::Property as PropertyDescription;

pub struct DmxProperty {
    property_handle: PropertyHandle,
    player: Arc<Mutex<Player>>,
    address: u8,
}

impl DmxProperty {
    pub fn new(property_handle: PropertyHandle, player: Arc<Mutex<Player>>, address: u8) -> Self {
        DmxProperty {
            property_handle,
            player,
            address,
        }
    }

    pub fn build_description(property_config: &PropertyConfig) -> PropertyDescription {
        PropertyDescription {
            at_type: Some(String::from("LevelProperty")),
            name: Some(property_config.id.clone()),
            title: Some(property_config.title.clone()),
            description: Some(property_config.title.clone()),
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
        }
    }
}

#[async_trait(?Send)]
impl Property for DmxProperty {
    fn borrow_property_handle(&mut self) -> &mut PropertyHandle {
        &mut self.property_handle
    }

    async fn on_update(&mut self, value: Value) -> Result<(), String> {
        let name = &self.property_handle.name;

        if let Value::Number(value) = value {
            let value = value
                .as_u64()
                .ok_or(format!("Value {} for {} is not an u64", value, name))?
                as u8;

            self.player
                .lock()
                .await
                .set(self.address as usize, vec![value])
                .map_err(|err| format!("Could not send DMX value: {}", err))
        } else {
            Err(format!("Value {} for {} is not a number", value, name))
        }
    }
}
