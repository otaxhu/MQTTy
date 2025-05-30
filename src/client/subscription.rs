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

use crate::client::MQTTyClientQos;

#[derive(Default, Clone)]
pub struct MQTTyClientSubscription {
    /// May contain wildcards
    pub topic_filter: String,

    pub qos: MQTTyClientQos,

    pub subscribed: bool,
    // TODO: For now, we are only supporting MQTT v3.x subscriptions, because
    // the v5 spec is too hard to understand :(
    //
    // This struct is missing all of the other options available for a MQTT v5
    // subscription
}
