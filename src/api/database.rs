/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlite::{Connection, Value};
use std::path::PathBuf;

pub struct Database {
    path: PathBuf,
    plugin_id: String,
}

impl Database {
    pub fn new(mut path: PathBuf, plugin_id: String) -> Self {
        path.push("db.sqlite3");

        Self { path, plugin_id }
    }

    pub fn load_config<T>(&self) -> T
    where
        T: DeserializeOwned,
    {
        let json = self.load_string();
        let config: T = serde_json::from_str(json.as_str()).expect("Could not parse config");
        config
    }

    pub fn load_string(&self) -> String {
        let key = self.key();
        let connection = self.open();

        let mut cursor = connection
            .prepare("SELECT value FROM settings WHERE key = ?")
            .expect("Could not select settings")
            .into_cursor();

        cursor
            .bind(&[Value::String(key)])
            .expect("Could not bind key");

        let row = cursor
            .next()
            .expect("Could not iterate over cursor")
            .expect("No config in the database found");

        row[0]
            .as_string()
            .expect("Value row is not a string")
            .to_owned()
    }

    pub fn save_config<T>(&self, t: &T)
    where
        T: Serialize,
    {
        let json = serde_json::to_string(t).expect("Could not serialize config");
        self.save_string(json);
    }

    pub fn save_string(&self, s: String) {
        let key = self.key();
        let connection = self.open();

        let mut statement = connection
            .prepare("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)")
            .expect("Could not prepare statement");

        statement
            .bind(1, key.as_str())
            .expect("Could not bind value");
        statement.bind(2, s.as_str()).expect("Could not bind value");
        statement.next().expect("Could not save config");
    }

    fn open(&self) -> Connection {
        sqlite::open(self.path.as_path()).expect("Could not open database")
    }

    fn key(&self) -> String {
        format!("addons.config.{}", self.plugin_id)
    }
}
