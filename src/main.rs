/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::adapter::DmxAdapter;
use crate::api::api_error::ApiError;
use crate::api::database::Database;
use crate::api::plugin::connect;
use crate::config::Config;
use std::path::PathBuf;

mod adapter;
mod api;
mod config;
mod device;
mod device_handler;
mod player;

#[tokio::main]
async fn main() {
    run().await.expect("Could not start adapter");
    println!("Exiting adapter");
}

async fn run() -> Result<(), ApiError> {
    let mut plugin = connect("dmx-adapter").await?;
    println!("Plugin registered");

    let config_path = PathBuf::from(plugin.user_profile.config_dir.clone());
    let database = Database::new(config_path, plugin.plugin_id.clone());
    let conf: Option<Config> = database.load_config()?;

    if let Some(conf) = conf {
        database.save_config(&conf)?;

        for adapter_config in conf.adapters {
            let id = adapter_config.id.clone();

            let title = adapter_config.title.clone();

            println!("Creating adapter '{}' ({})", title, id);

            let mut dmx_adapter = DmxAdapter::new();

            let mut adapter = plugin.create_adapter(&id, &title).await?;

            if let Err(err) = dmx_adapter.init(&mut adapter, adapter_config).await {
                plugin
                    .fail(format!("Failed to initialize adapter: {}", err))
                    .await?;
            }
        }
    }

    plugin.event_loop().await;

    Ok(())
}
