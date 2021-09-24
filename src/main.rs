/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use crate::adapter::DmxAdapter;
use crate::config::Config;
use gateway_addon_rust::{api_error::ApiError, database::Database, plugin::connect};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::path::PathBuf;

mod adapter;
mod config;
mod device;
mod player;
mod property;

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap();

    if let Err(err) = run().await {
        log::error!("Could not start adapter: {}", err);
    }

    log::info!("Exiting adapter");
}

async fn run() -> Result<(), ApiError> {
    let mut plugin = connect("dmx-adapter").await?;
    log::debug!("Plugin registered");

    let config_path = PathBuf::from(plugin.user_profile.config_dir.clone());
    let database = Database::new(config_path, plugin.plugin_id.clone());
    let conf: Option<Config> = database.load_config()?;

    if let Some(conf) = conf {
        database.save_config(&conf)?;

        for adapter_config in conf.adapters {
            let id = adapter_config.id.clone();

            let title = adapter_config.title.clone();

            log::debug!("Creating adapter '{}' ({})", title, id);

            let adapter = plugin
                .create_adapter(&id, &title, |adapter_handle| {
                    DmxAdapter::new(adapter_handle)
                })
                .await?;

            let result = adapter.lock().await.init(adapter_config).await;

            if let Err(err) = result {
                plugin
                    .fail(format!("Failed to initialize adapter: {}", err))
                    .await?;
            }
        }
    }

    plugin.event_loop().await;

    Ok(())
}
