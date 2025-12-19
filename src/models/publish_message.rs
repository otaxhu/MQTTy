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

use crate::client::MQTTyClientQos;
use crate::content_type::MQTTyContentType;

/// This represents an MQTT message for publishing purposes, for this application.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct MQTTyPublishMessageModel {
    pub topic: String,
    pub qos: MQTTyClientQos,
    pub retained: bool,
    pub content_type: MQTTyContentType,
    pub user_properties: Vec<(String, String)>,
    pub body: Vec<u8>,
}
