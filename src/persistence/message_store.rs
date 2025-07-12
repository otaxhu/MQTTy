// Copyright (c) 2025 Oscar Pernia
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::fs;
use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use base64::Engine;
use chrono::{DateTime, Local};
use directories::ProjectDirs;
use sha2::{Digest, Sha256};

use crate::client::{MQTTyClientConnection, MQTTyClientMessage};

pub struct MQTTyMessageStore {
    connection: MQTTyClientConnection,
    hash: String,
}

impl MQTTyMessageStore {
    pub fn new(connection: MQTTyClientConnection) -> Self {
        let hash = hex::encode(
            Sha256::new()
                .chain_update(&connection.client_id)
                .chain_update(&connection.url)
                .finalize(),
        );

        Self { connection, hash }
    }

    fn get_log_dir() -> PathBuf {
        ProjectDirs::from("io.github", "otaxhu", "MQTTy")
            .expect("No valid data directory found")
            .data_dir()
            .join("logs")
    }

    pub fn store_message(&self, msg: &MQTTyClientMessage) -> std::io::Result<()> {
        let log_dir = Self::get_log_dir().join(&self.hash);

        fs::create_dir_all(&log_dir)?;

        let meta_path = log_dir.join("meta.json");
        if !meta_path.exists() {
            let mut meta_file = File::create(&meta_path)?;
            let meta = serde_json::json!({
                "client_id": self.connection.client_id,
                "url": self.connection.url,
                "username": self.connection.username,
            });
            serde_json::to_writer_pretty(&mut meta_file, &meta)?;
        }

        let parsed_date = msg.timestamp().parse::<DateTime<Local>>().map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Failed to parse datetime from MQTTyClientMessage.timestamp: {:?}",
                    e,
                ),
            )
        })?;
        let log_path = log_dir.join(parsed_date.format("%Y-%m-%d.log").to_string());
        let mut log_file =
            BufWriter::new(File::options().create(true).append(true).open(log_path)?);

        const BASE64_ENCODER: base64::engine::GeneralPurpose = base64::engine::GeneralPurpose::new(
            &base64::alphabet::URL_SAFE,
            base64::engine::GeneralPurposeConfig::new().with_encode_padding(false),
        );

        let entry = serde_json::json!({
            "timestamp": msg.timestamp(),
            "topic": msg.topic(),
            "qos": msg.qos(),
            "version": msg.mqtt_version(),
            "retained": msg.retained(),
            "content_type": msg.content_type(),
            "user_properties": msg.user_properties(),
            "body": BASE64_ENCODER.encode(msg.body()),
        });

        writeln!(log_file, "{}", entry.to_string())?;
        Ok(())
    }

    pub fn get_logs(&self) -> std::io::Result<Vec<PathBuf>> {
        let logs_dir = Self::get_log_dir().join(&self.hash);
        let mut logs = vec![];

        if logs_dir.exists() {
            for entry in fs::read_dir(logs_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    logs.push(path);
                }
            }
        }

        Ok(logs)
    }
}
