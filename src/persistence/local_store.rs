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

use std::path::PathBuf;

use directories::ProjectDirs;
use indoc::indoc; // Used in order to generate properly indented SQL queries
use rusqlite::OptionalExtension;

use crate::client::{
    MQTTyClientConnection, MQTTyClientMessage, MQTTyClientQos, MQTTyClientVersion,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Project directory not found (XDG dirs)")]
    NoProjectDir,
}

pub type Result<T> = std::result::Result<T, Error>;

/// This struct handles local storage using a SQLite database for MQTT messages
/// and MQTT connections, but mostly the former.
pub struct MQTTyLocalStore {
    sql_conn: rusqlite::Connection,
}

impl MQTTyLocalStore {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        let sql_conn = rusqlite::Connection::open(db_path)?;

        sql_conn.execute_batch(indoc! {
            r#"
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS mqtt_connections (
                    id         INTEGER PRIMARY KEY
                    ,
                    -- We are not storing all of the fields from a mqtt connection,
                    -- only storing identity fields, which in this case are client_id and url
                    client_id  TEXT NOT NULL
                    ,
                    url        TEXT NOT NULL
                    ,

                    -- Soft-deletable
                    deleted  BOOLEAN NOT NULL DEFAULT FALSE
                    ,

                    UNIQUE (client_id, url)
                );

                CREATE TABLE IF NOT EXISTS mqtt_messages (
                    id               INTEGER PRIMARY KEY
                    ,
                    connection_id    INTEGER NOT NULL REFERENCES mqtt_connections(id) ON DELETE CASCADE
                    ,
                    timestamp        TEXT NOT NULL
                    ,
                    topic            TEXT NOT NULL
                    ,
                    qos              INTEGER NOT NULL
                    ,
                    version          TEXT NOT NULL
                    ,
                    retained         BOOLEAN NOT NULL
                    ,
                    content_type     TEXT
                    ,
                    user_properties  TEXT NOT NULL -- JSON format, empty value should be empty array []
                    ,
                    body             BLOB NOT NULL
                );
            "#
        })?;

        Ok(Self { sql_conn })
    }

    /// TODO: inexistant feature "Workspaces" should call this function
    ///
    /// This function is supposed to be called when the application starts
    /// or by user interaction, it syncs the user Workspace file
    /// (only the connections are needed and should be passed to `conns`)
    /// with this database.
    ///
    /// The connections that are both in `conns` and the `mqtt_connections` table
    /// are automatically updated and the field `deleted` is reset to `FALSE`.
    /// Connections that exist in the database but not in `conns` are soft-deleted.
    ///
    /// If a connection from `conns` doesn't exist in the database, it is inserted.
    pub fn sync_connections(&self, conns: &[MQTTyClientConnection]) -> Result<()> {
        todo!()
    }

    fn get_db_path() -> Result<PathBuf> {
        ProjectDirs::from("io.github", "otaxhu", "MQTTy")
            .ok_or(Error::NoProjectDir)
            .map(|d| d.data_dir().join("messages.db"))
    }

    /// Returns the row id, and whether a new row was inserted
    fn get_or_insert_connection(&self, conn: &MQTTyClientConnection) -> Result<(i64, bool)> {
        let connection_id = self
            .sql_conn
            .query_row(
                r#"SELECT id FROM mqtt_connections WHERE client_id = ?1 AND url = ?2"#,
                rusqlite::params![conn.client_id, conn.url],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;

        let found = connection_id.is_some();

        let connection_id = match connection_id {
            Some(id) => id,
            None => {
                self.sql_conn.execute(
                    indoc! {
                        r#"
                            INSERT INTO mqtt_connections
                                (client_id, url)
                            VALUES
                                (?1, ?2)
                        "#
                    },
                    rusqlite::params![conn.client_id, conn.url],
                )?;
                self.sql_conn.last_insert_rowid()
            }
        };

        Ok((connection_id, !found))
    }

    pub fn store_message(
        &self,
        conn: &MQTTyClientConnection,
        msg: &MQTTyClientMessage,
    ) -> Result<()> {
        let (conn_id, _) = self.get_or_insert_connection(conn)?;

        let timestamp = msg.timestamp();
        let topic = msg.topic();
        let qos = match msg.qos() {
            MQTTyClientQos::Qos0 => 0,
            MQTTyClientQos::Qos1 => 1,
            MQTTyClientQos::Qos2 => 2,
        };
        let version = match msg.mqtt_version() {
            MQTTyClientVersion::V3X => "3.x",
            MQTTyClientVersion::V5 => "5",
        };
        let content_type = msg.content_type();
        let user_properties = serde_json::to_string(&msg.user_properties())?;
        let retained = msg.retained();
        let body = msg.body();

        self.sql_conn.execute(
            indoc! {
                r#"
                    INSERT INTO mqtt_messages (
                        connection_id, timestamp, topic, qos, version,
                        retained, content_type, user_properties, body
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#
            },
            rusqlite::params![
                conn_id,
                timestamp,
                topic,
                qos,
                version,
                retained,
                content_type,
                user_properties,
                body
            ],
        )?;

        Ok(())
    }

    /// Returns a tuple `(messages, next_cursor)` for the given connection,
    /// where `messages` is a list of messages ordered from newest to oldest,
    /// and `next_cursor` is an optional cursor value that can be used as the `cursor`
    /// parameter in a subsequent call to continue paginating older messages.
    /// If there are no more messages, `next_cursor` will be `None`.
    ///
    /// The number of messages returned will not exceed `limit`.
    pub fn get_recent_messages_for_connection(
        &self,
        conn: &MQTTyClientConnection,
        cursor_id: Option<i64>,
        limit: i64,
    ) -> Result<(Vec<MQTTyClientMessage>, Option<i64>)> {
        let mut query = String::from(indoc! {
            r#"
                SELECT id, timestamp, topic, qos, version, retained, content_type, user_properties, body
                FROM mqtt_messages
                WHERE connection_id = (
                    SELECT id FROM mqtt_connections WHERE client_id = ?1 AND url = ?2
                )
            "#
        });

        let limit_param = limit + 1;

        let mut params: Vec<&dyn rusqlite::ToSql> = vec![&conn.client_id, &conn.url, &limit_param];

        if let Some(ref id) = cursor_id {
            query.push_str(" AND id <= ?4");
            params.push(id);
        }

        query.push_str(" ORDER BY id DESC LIMIT ?3");

        let mut stmt = self.sql_conn.prepare(&query)?;
        let mut rows = stmt
            .query_map(params.as_slice(), |row| {
                let id = row.get::<_, i64>(0)?;
                let timestamp = row.get::<_, String>(1)?;
                let topic = row.get::<_, String>(2)?;
                let qos = match row.get::<_, i64>(3)? {
                    0 => MQTTyClientQos::Qos0,
                    1 => MQTTyClientQos::Qos1,
                    2 => MQTTyClientQos::Qos2,
                    n => {
                        return Err(rusqlite::Error::FromSqlConversionFailure(
                            3,
                            rusqlite::types::Type::Integer,
                            format!("Invalid qos: {n}").into(),
                        ))
                    }
                };
                let version = match row.get::<_, String>(4)?.as_str() {
                    "3.x" => MQTTyClientVersion::V3X,
                    "5" => MQTTyClientVersion::V5,
                    v => {
                        return Err(rusqlite::Error::FromSqlConversionFailure(
                            4,
                            rusqlite::types::Type::Text,
                            format!("Invalid version: {v}").into(),
                        ))
                    }
                };
                let retained = row.get::<_, bool>(5)?;
                let content_type = row.get::<_, Option<String>>(6)?;
                let user_properties: Vec<(String, String)> =
                    if let Some(json) = row.get::<_, Option<String>>(7)? {
                        serde_json::from_str(&json).map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                7,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?
                    } else {
                        Default::default()
                    };
                let body = row.get::<_, Vec<u8>>(8)?;

                let message = MQTTyClientMessage::new();
                message.set_timestamp(timestamp);
                message.set_topic(topic);
                message.set_qos(qos);
                message.set_mqtt_version(version);
                message.set_retained(retained);
                message.set_content_type(content_type);
                message.set_user_properties(&user_properties);
                message.set_body(&body);

                Ok((message, id))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let next_cursor_id = rows.get(limit as usize).map(|r| r.1);
        if next_cursor_id.is_some() {
            rows.pop();
        }

        Ok((rows.into_iter().map(|r| r.0).collect(), next_cursor_id))
    }
}
