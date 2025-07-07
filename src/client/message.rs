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

use std::cell::{Cell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use crate::client::{MQTTyClientQos, MQTTyClientVersion};

mod imp {

    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::MQTTyClientMessage)]
    pub struct MQTTyClientMessage {
        #[property(get, set)]
        topic: RefCell<String>,

        #[property(get, set, builder(Default::default()))]
        qos: Cell<MQTTyClientQos>,

        #[property(get, set, builder(Default::default()))]
        mqtt_version: Cell<MQTTyClientVersion>,

        #[property(get, set)]
        retained: Cell<bool>,

        #[property(get, set, nullable)]
        content_type: RefCell<Option<String>>,

        #[property(get, set)]
        timestamp: RefCell<String>,

        pub user_properties: RefCell<Vec<(String, String)>>,

        pub body: RefCell<Vec<u8>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyClientMessage {
        const NAME: &'static str = "MQTTyClientMessage";

        type Type = super::MQTTyClientMessage;

        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyClientMessage {}
}

glib::wrapper! {
    /// This type works as a model, that carries all of the data related to a
    /// publish/subscribed MQTT message
    ///
    /// Implements serialization and deserialization of MQTT messages
    ///
    /// TODO: add serde functions
    pub struct MQTTyClientMessage(ObjectSubclass<imp::MQTTyClientMessage>);
}

impl MQTTyClientMessage {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn body(&self) -> Vec<u8> {
        self.imp().body.borrow().clone()
    }

    pub fn set_body(&self, body: &[u8]) {
        let mut v = self.imp().body.borrow_mut();
        v.clear();
        v.extend_from_slice(body);
    }

    pub fn user_properties(&self) -> Vec<(String, String)> {
        self.imp().user_properties.borrow().clone()
    }

    pub fn set_user_properties(&self, user_properties: &[(String, String)]) {
        let mut v = self.imp().user_properties.borrow_mut();
        v.clear();
        v.extend_from_slice(user_properties);
    }
}
