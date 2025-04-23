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

use gettextrs::pgettext;
use gtk::glib;

#[derive(Default, Clone, Copy, glib::Enum)]
#[enum_type(name = "MQTTyContentType")]
pub enum MQTTyContentType {
    #[default]
    None,
    Json,
    Xml,
    Raw,
}

impl MQTTyContentType {
    pub fn listed() -> &'static [MQTTyContentType] {
        &[
            MQTTyContentType::None,
            MQTTyContentType::Json,
            MQTTyContentType::Xml,
            MQTTyContentType::Raw,
        ]
    }

    pub fn translated(&self) -> String {
        match self {
            MQTTyContentType::None => pgettext("body content type", "(none)"),
            MQTTyContentType::Json => pgettext("body content type", "JSON"),
            MQTTyContentType::Xml => pgettext("body content type", "XML"),
            MQTTyContentType::Raw => pgettext("body content type", "Raw"),
        }
    }
}
