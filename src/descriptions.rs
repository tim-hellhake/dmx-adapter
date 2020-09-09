/**
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDescription {
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<String>>,
    pub id: String,
    pub title: String,
    pub description: String,
    pub properties: HashMap<String, PropertyDescription>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PropertyDescription {
    #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
    pub property_type: Option<String>,
    pub name: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub value_enum: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
    pub value: Value,
}
