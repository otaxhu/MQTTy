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
use gtk::glib::subclass::Signal;

use crate::widgets::MQTTyKeyValueRow;

mod imp {

    use std::sync::LazyLock;

    use super::*;

    #[derive(Default, glib::Properties, Debug)]
    #[properties(wrapper_type = super::MQTTyKeyValue)]
    pub struct MQTTyKeyValue {
        #[property(get, set)]
        active: Cell<bool>,

        #[property(get, set)]
        key: RefCell<String>,

        #[property(get, set)]
        value: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyKeyValue {
        const NAME: &'static str = "MQTTyKeyValue";

        type Type = super::MQTTyKeyValue;

        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyKeyValue {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let changed_callback = |obj: &Self::Type| {
                println!("key_value: {:?}", obj.imp());
                obj.emit_by_name::<()>("changed", &[]);
            };

            obj.connect_key_notify(changed_callback.clone());
            obj.connect_value_notify(changed_callback.clone());
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> =
                LazyLock::new(|| vec![Signal::builder("changed").build()]);
            &*SIGNALS
        }
    }
}

glib::wrapper! {
    /// This type is used for data serialization and deserialization for MQTTyKeyValueRow, but it doesn't
    /// handles presentation
    ///
    /// TODO: add serde functions
    pub struct MQTTyKeyValue(ObjectSubclass<imp::MQTTyKeyValue>);
}

impl Default for MQTTyKeyValue {
    fn default() -> Self {
        Self::new("", "", false)
    }
}

impl MQTTyKeyValue {
    fn new(key: &str, value: &str, active: bool) -> Self {
        glib::Object::builder()
            .property("key", key)
            .property("value", value)
            .property("active", active)
            .build()
    }
}

impl From<MQTTyKeyValueRow> for MQTTyKeyValue {
    fn from(value: MQTTyKeyValueRow) -> Self {
        Self::new(&value.key(), &value.value(), value.active())
    }
}
