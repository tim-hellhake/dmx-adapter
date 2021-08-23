/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Failed to send message")]
    Send(#[source] tungstenite::Error),

    #[error("Failed to serialize message")]
    Serialization(#[source] serde_json::Error),

    #[error("Device {device_id} has no property called {property_name}")]
    PropertyNotFound {
        device_id: String,
        property_name: String,
    },
}
