/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::config::RgbProperty as RgbPropertyConfig;
use crate::player::Player;
use async_trait::async_trait;
use gateway_addon_rust::property::{Property, PropertyHandle};
use hex_color::HexColor;
use serde_json::value::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use webthings_gateway_ipc_types::Property as PropertyDescription;

pub struct RgbProperty {
    property_handle: PropertyHandle,
    player: Arc<Mutex<Player>>,
    config: RgbPropertyConfig,
}

impl RgbProperty {
    pub fn new(
        property_handle: PropertyHandle,
        player: Arc<Mutex<Player>>,
        config: RgbPropertyConfig,
    ) -> Self {
        RgbProperty {
            property_handle,
            player,
            config,
        }
    }

    pub fn build_description(rgb_property_config: &RgbPropertyConfig) -> PropertyDescription {
        PropertyDescription {
            at_type: Some(String::from("ColorProperty")),
            name: Some(rgb_property_config.id.clone()),
            title: Some(rgb_property_config.title.clone()),
            description: Some(rgb_property_config.title.clone()),
            type_: String::from("string"),
            unit: None,
            enum_: None,
            links: None,
            minimum: None,
            maximum: None,
            multiple_of: None,
            read_only: Some(false),
            value: None,
            visible: Some(true),
        }
    }
}

#[async_trait(?Send)]
impl Property for RgbProperty {
    fn borrow_property_handle(&mut self) -> &mut PropertyHandle {
        &mut self.property_handle
    }

    async fn on_update(&mut self, value: Value) -> Result<(), String> {
        let name = &self.property_handle.name;

        if let Value::String(value) = value {
            let color: HexColor = value
                .parse()
                .map_err(|err| format!("Could not parse color string {}: {}", value, err))?;

            let player = self.player.lock().await;

            player
                .set(vec![
                    (self.config.red, color.r),
                    (self.config.green, color.g),
                    (self.config.blue, color.b),
                ])
                .await
                .map_err(|err| format!("Could not set DMX values: {}", err))
        } else {
            Err(format!("Value {} for {} is not a string", value, name))
        }
    }
}
