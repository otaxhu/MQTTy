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

use std::cell::RefCell;

use adw::subclass::prelude::*;
use gtk::glib;
use gtk::glib::variant::{FromVariant, StaticVariantType};
use gtk::prelude::*;

mod imp {

    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::MQTTySettingConnection)]
    pub struct MQTTySettingConnection {
        #[property(get, set)]
        url: RefCell<String>,

        #[property(get, set)]
        topic: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySettingConnection {
        const NAME: &'static str = "MQTTyOpenConnection";

        type Type = super::MQTTySettingConnection;

        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTySettingConnection {}
}

glib::wrapper! {
    pub struct MQTTySettingConnection(ObjectSubclass<imp::MQTTySettingConnection>);
}

impl MQTTySettingConnection {
    pub fn new(url: &String, topic: &String) -> Self {
        glib::Object::builder()
            .property("url", url)
            .property("topic", topic)
            .build()
    }
}

impl Default for MQTTySettingConnection {
    fn default() -> Self {
        Self::new(&"".to_string(), &"".to_string())
    }
}

const VARIANT_TYPE: &str = "(ss)";

impl StaticVariantType for MQTTySettingConnection {
    fn static_variant_type() -> std::borrow::Cow<'static, gtk::glib::VariantTy> {
        glib::VariantTy::new(VARIANT_TYPE).unwrap().into()
    }
}

/// Indexes mapping:
/// - 0 <-> url: URL connection
/// - 1 <-> topic: MQTT topic
type MQTTySettingConnectionTuple = (String, String);

impl FromVariant for MQTTySettingConnection {
    fn from_variant(variant: &gtk::glib::Variant) -> Option<Self> {
        let tuple = variant.get::<MQTTySettingConnectionTuple>();
        if tuple.is_none() {
            tracing::error!(
                "Could not convert from variant with format '{}', expected '{}'",
                variant.type_(),
                VARIANT_TYPE
            );
        }

        tuple.map(|tuple| tuple.into())
    }
}

impl From<MQTTySettingConnectionTuple> for MQTTySettingConnection {
    fn from(value: MQTTySettingConnectionTuple) -> Self {
        Self::new(&value.0, &value.1)
    }
}

impl From<MQTTySettingConnection> for MQTTySettingConnectionTuple {
    fn from(value: MQTTySettingConnection) -> Self {
        (value.url(), value.topic())
    }
}

impl From<MQTTySettingConnection> for glib::Variant {
    fn from(value: MQTTySettingConnection) -> Self {
        // Converts to tuple then to GVariant
        Into::<MQTTySettingConnectionTuple>::into(value).into()
    }
}
