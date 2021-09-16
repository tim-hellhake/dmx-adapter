/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::api::device::{Device, DeviceHandle};
use crate::config::Device as DeviceConfig;
use crate::player::Player;
use async_trait::async_trait;
use serde_json::value::Value;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::sync::Mutex;
use webthings_gateway_ipc_types::{Device as DeviceDescription, Property as PropertyDescription};

pub struct DmxDevice {
    device_handle: DeviceHandle,
    player: Arc<Mutex<Player>>,
    property_addresses: HashMap<String, u8>,
}

impl DmxDevice {
    pub fn new(
        device_config: DeviceConfig,
        device_handle: DeviceHandle,
        player: Arc<Mutex<Player>>,
    ) -> Self {
        Self {
            device_handle,
            player,
            property_addresses: DmxDevice::build_property_addresses(device_config),
        }
    }

    fn build_property_addresses(device_config: DeviceConfig) -> HashMap<String, u8> {
        let mut property_addresses = HashMap::new();

        for property_config in device_config.properties {
            property_addresses.insert(property_config.id.clone(), property_config.address);
        }

        property_addresses
    }

    pub fn build_description(device_config: &DeviceConfig) -> DeviceDescription {
        let mut property_descriptions = BTreeMap::new();

        for property_config in &device_config.properties {
            let id = property_config.id.clone();

            log::debug!(
                "Creating property '{}' ({}) for '{}'",
                property_config.title,
                id,
                device_config.title
            );

            let description = PropertyDescription {
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
            };

            property_descriptions.insert(id.clone(), description);
        }

        DeviceDescription {
            at_context: None,
            at_type: Some(vec![]),
            id: device_config.id.clone(),
            title: Some(String::from("Dmx")),
            description: Some(String::from("A dmx light")),
            properties: Some(property_descriptions),
            actions: None,
            events: None,
            links: None,
            base_href: None,
            pin: None,
            credentials_required: None,
        }
    }
}

#[async_trait(?Send)]
impl Device for DmxDevice {
    async fn on_property_updated(&mut self, name: &str, value: Value) -> Result<(), String> {
        let id = self.device_handle.description.id.clone();

        let address = self.property_addresses.get(name).ok_or(format!(
            "Cannot find property address for '{}' in '{}'",
            name, id
        ))?;

        if let Value::Number(value) = value {
            let value = value
                .as_u64()
                .ok_or(format!("Value {} for {} is not an u64", value, name))?
                as u8;

            self.player
                .lock()
                .await
                .set(*address as usize, vec![value])
                .map_err(|err| format!("Could not send DMX value: {}", err))?;

            self.device_handle
                .set_property_value(name, Value::from(value))
                .await
                .map_err(|err| format!("Could not set value for property {}: {}", name, err))
        } else {
            Err(format!("Value {} for {} is not a number", value, name))
        }
    }
}
