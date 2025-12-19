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

use crate::models::{MQTTyConnectionModel, MQTTyConnectionSessionModel, MQTTySubscriptionModel};

/// This model represents an MQTT connection only suitable for handling MQTT
/// subscriptions, for this application.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct MQTTyConnectionSubscriptionModel {
    connection: MQTTyConnectionSessionModel,

    pub subscriptions: Vec<MQTTySubscriptionModel>,
}

impl AsRef<MQTTyConnectionSessionModel> for MQTTyConnectionSubscriptionModel {
    fn as_ref(&self) -> &MQTTyConnectionSessionModel {
        &self.connection
    }
}

impl AsRef<MQTTyConnectionModel> for MQTTyConnectionSubscriptionModel {
    fn as_ref(&self) -> &MQTTyConnectionModel {
        self.connection.as_ref()
    }
}

impl AsMut<MQTTyConnectionSessionModel> for MQTTyConnectionSubscriptionModel {
    fn as_mut(&mut self) -> &mut MQTTyConnectionSessionModel {
        &mut self.connection
    }
}

impl AsMut<MQTTyConnectionModel> for MQTTyConnectionSubscriptionModel {
    fn as_mut(&mut self) -> &mut MQTTyConnectionModel {
        self.connection.as_mut()
    }
}
