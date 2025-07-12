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
use std::sync::LazyLock;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::glib::subclass::Signal;

use crate::client::{MQTTyClientConnection, MQTTyClientSubscription};

mod imp {

    use super::*;

    #[derive(Default)]
    pub struct MQTTyClientSubscriptionsData {
        pub connection: RefCell<MQTTyClientConnection>,

        pub subscriptions: RefCell<Vec<MQTTyClientSubscription>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyClientSubscriptionsData {
        const NAME: &'static str = "MQTTyClientSubscriptionsData";

        type Type = super::MQTTyClientSubscriptionsData;

        type ParentType = glib::Object;
    }

    impl ObjectImpl for MQTTyClientSubscriptionsData {
        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> =
                LazyLock::new(|| vec![Signal::builder("changed").build()]);
            &*SIGNALS
        }
    }
}

glib::wrapper! {
    /// This is a serializable struct for holding the data necessary to connect,
    /// reconnect and subscribe to a list of topics
    ///
    /// TODO: Add serde functions
    pub struct MQTTyClientSubscriptionsData(ObjectSubclass<imp::MQTTyClientSubscriptionsData>);
}

impl MQTTyClientSubscriptionsData {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_subscriptions(&self, subscriptions: &[MQTTyClientSubscription]) {
        let mut v = self.imp().subscriptions.borrow_mut();
        v.clear();
        v.extend_from_slice(subscriptions);
        self.emit_by_name::<()>("changed", &[]);
    }

    pub fn subscriptions(&self) -> Vec<MQTTyClientSubscription> {
        self.imp().subscriptions.borrow().clone()
    }

    pub fn connection(&self) -> MQTTyClientConnection {
        self.imp().connection.borrow().clone()
    }

    pub fn set_connection(&self, conn: &MQTTyClientConnection) {
        self.imp().connection.borrow_mut().clone_from(conn);
        self.emit_by_name::<()>("changed", &[]);
    }

    pub fn connect_changed(&self, cb: impl Fn(&Self) + 'static) -> glib::SignalHandlerId {
        self.connect_closure("changed", false, glib::closure_local!(|o: &Self| cb(o)))
    }
}
