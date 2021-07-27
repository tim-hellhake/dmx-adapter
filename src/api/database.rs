/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.*
 */
use sqlite::{Connection, Value};
use std::env;
use std::path::{Path, PathBuf};

pub fn load_string(path: PathBuf) -> String {
    let key = key();
    let connection = open(path);

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

    return row[0]
        .as_string()
        .expect("Value row is not a string")
        .to_string();
}

pub fn save_string(path: PathBuf, s: String) {
    let key = key();
    let connection = open(path);

    let mut statement = connection
        .prepare("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)")
        .expect("Could not prepare statement");

    statement
        .bind(1, key.as_str())
        .expect("Could not bind value");
    statement.bind(2, s.as_str()).expect("Could not bind value");
    statement.next().expect("Could not save config");
}

fn open(mut path: PathBuf) -> Connection {
    path.push("db.sqlite3");

    sqlite::open(path).expect("Could not open database")
}

fn key() -> String {
    let arg = env::args()
        .nth(1)
        .expect("Expects addon path as first argument");

    let path = Path::new(&arg);

    let addon_dir = path
        .file_stem()
        .expect("Could not extract addon name from path");

    return format!(
        "addons.config.{}",
        addon_dir
            .to_str()
            .expect("Could not convert add name to str")
    );
}
