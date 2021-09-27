/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::config::Device as DeviceConfig;
use crate::player::Player;
use crate::property::DmxProperty;
use crate::rgb_property::RgbProperty;
use async_trait::async_trait;
use gateway_addon_rust::device::{Device, DeviceHandle};

use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use webthings_gateway_ipc_types::Device as DeviceDescription;

pub struct DmxDevice {
    device_handle: DeviceHandle,
}

impl DmxDevice {
    pub fn new(
        device_config: DeviceConfig,
        mut device_handle: DeviceHandle,
        player: Arc<Mutex<Player>>,
    ) -> Self {
        for property_config in &device_config.properties {
            let id = property_config.id.clone();

            log::debug!(
                "Creating property '{}' ({}) for '{}'",
                property_config.title,
                id,
                device_config.title
            );

            let description = DmxProperty::build_description(property_config);

            device_handle.add_property(
                property_config.id.clone(),
                description,
                |property_handle| {
                    DmxProperty::new(property_handle, player.clone(), property_config.address)
                },
            );
        }

        for property_config in device_config.rgb_properties {
            let id = property_config.id.clone();

            log::debug!(
                "Creating rgb property '{}' ({}) for '{}'",
                property_config.title,
                id,
                device_config.title
            );

            let description = RgbProperty::build_description(&property_config);

            device_handle.add_property(
                property_config.id.clone(),
                description,
                |property_handle| {
                    RgbProperty::new(property_handle, player.clone(), property_config)
                },
            );
        }

        Self { device_handle }
    }

    pub fn build_description(device_config: &DeviceConfig) -> DeviceDescription {
        let mut property_descriptions = BTreeMap::new();

        for property_config in &device_config.properties {
            let id = property_config.id.clone();

            log::debug!(
                "Building property description '{}' ({}) for '{}'",
                property_config.title,
                id,
                device_config.title
            );

            let description = DmxProperty::build_description(property_config);

            property_descriptions.insert(id.clone(), description);
        }

        for property_config in &device_config.rgb_properties {
            let id = property_config.id.clone();

            log::debug!(
                "Building rgb property description '{}' ({}) for '{}'",
                property_config.title,
                id,
                device_config.title
            );

            let description = RgbProperty::build_description(property_config);

            property_descriptions.insert(id.clone(), description);
        }

        let mut device_types = Vec::new();

        if !device_config.rgb_properties.is_empty() {
            device_types.push(String::from("ColorControl"));
        }

        DeviceDescription {
            at_context: None,
            at_type: Some(device_types),
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
    fn borrow_device_handle(&mut self) -> &mut DeviceHandle {
        &mut self.device_handle
    }
}
