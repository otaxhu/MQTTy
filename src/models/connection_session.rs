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

use serde::{Deserialize, Serialize};

use super::MQTTyConnectionModel;

/// This model represents an MQTT connection, but with associated fields
/// that help to reproduce an MQTT session.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct MQTTyConnectionSessionModel {
    pub connection: MQTTyConnectionModel,
    pub client_id: String,

    /// The user choice for this client to be connected.
    pub user_connected: bool,

    /// This field is an abstraction of the clean_start flag in a MQTT client,
    /// designed specifically for this application.
    ///
    /// ## Explanation:
    ///
    /// If the user doesn't want to receive old messages after connecting and it was
    /// purposefully disconnected (by fully closing the app or setting user_connected to `false`,
    /// it doesn't count if it was because connection lost), he can set this field to
    /// `true` and when the client wrapper gets connected, it will first wipe the
    /// current session, and then finally reconnect with clean_start set always to `false`.
    pub wipe_queue_on_connect: bool,
}

impl AsRef<MQTTyConnectionModel> for MQTTyConnectionSessionModel {
    fn as_ref(&self) -> &MQTTyConnectionModel {
        &self.connection
    }
}

impl AsMut<MQTTyConnectionModel> for MQTTyConnectionSessionModel {
    fn as_mut(&mut self) -> &mut MQTTyConnectionModel {
        &mut self.connection
    }
}
