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

use crate::models::{MQTTyConnectionModel, MQTTyPublishMessageModel};

/// This model represents an MQTT connection only suitable for sending PUBLISH
/// packets, for this application.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct MQTTyConnectionPublishModel {
    connection: MQTTyConnectionModel,

    publish_message: MQTTyPublishMessageModel,
}

impl AsRef<MQTTyConnectionModel> for MQTTyConnectionPublishModel {
    fn as_ref(&self) -> &MQTTyConnectionModel {
        &self.connection
    }
}

impl AsMut<MQTTyConnectionModel> for MQTTyConnectionPublishModel {
    fn as_mut(&mut self) -> &mut MQTTyConnectionModel {
        &mut self.connection
    }
}

impl AsRef<MQTTyPublishMessageModel> for MQTTyConnectionPublishModel {
    fn as_ref(&self) -> &MQTTyPublishMessageModel {
        &self.publish_message
    }
}

impl AsMut<MQTTyPublishMessageModel> for MQTTyConnectionPublishModel {
    fn as_mut(&mut self) -> &mut MQTTyPublishMessageModel {
        &mut self.publish_message
    }
}
