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
use std::sync::LazyLock;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::Signal;
use gtk::glib;

use crate::client::MQTTyClientQos;

mod imp {

    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::MQTTyClientSubscription)]
    pub struct MQTTyClientSubscription {
        /// May contain wildcards
        #[property(get, set)]
        pub topic_filter: RefCell<String>,

        #[property(get, set, builder(Default::default()))]
        pub qos: Cell<MQTTyClientQos>,

        #[property(get, set)]
        pub subscribed: Cell<bool>,
        // TODO: For now, we are only supporting MQTT v3.x subscriptions, because
        // the v5 spec is too hard to understand :(
        //
        // This struct is missing all of the other options available for a MQTT v5
        // subscription
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyClientSubscription {
        const NAME: &'static str = "MQTTyClientSubscription";

        type Type = super::MQTTyClientSubscription;

        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyClientSubscription {
        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> =
                LazyLock::new(|| vec![Signal::builder("user-changed").build()]);
            &*SIGNALS
        }
    }
}

glib::wrapper! {
    /// Signals:
    ///
    /// - "::user-changed", this signal should be emitted by widgets when it is considered
    /// that the user changed some/all of the object's props, we are not using the
    /// ::notify signals because a user could modify multiple props, causing multiple emissions.
    /// Components should connect to this signal for example to subscribe/unsubscribe/resubscribe
    /// the client to the new modified subscriptions.
    pub struct MQTTyClientSubscription(ObjectSubclass<imp::MQTTyClientSubscription>);
}

impl MQTTyClientSubscription {
    pub fn connect_user_changed(&self, cb: impl Fn(&Self) + 'static) -> glib::SignalHandlerId {
        self.connect_closure(
            "user-changed",
            false,
            glib::closure_local!(|o: &Self| cb(o)),
        )
    }

    pub fn emit_user_changed(&self) {
        self.emit_by_name::<()>("user-changed", &[]);
    }
}
