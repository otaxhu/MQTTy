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
use glib::subclass::Signal;
use gtk::glib;

use crate::client::MQTTyClientQos;
use crate::models::MQTTySubscriptionModel;

mod imp {

    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::MQTTySubscriptionMessagesSubscription)]
    pub struct MQTTySubscriptionMessagesSubscription {
        #[property(name = "topic-filter", get, member = topic_filter, type = String)]
        #[property(name = "qos", get, member = qos, type = MQTTyClientQos, builder(Default::default()))]
        #[property(name = "user-subscribed", get, set, member = user_subscribed, type = bool)]
        pub subscription_model: RefCell<MQTTySubscriptionModel>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionMessagesSubscription {
        const NAME: &'static str = "MQTTySubscriptionMessagesSubscription";

        type Type = super::MQTTySubscriptionMessagesSubscription;

        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTySubscriptionMessagesSubscription {
        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> =
                LazyLock::new(|| vec![Signal::builder("changed").build()]);
            &*SIGNALS
        }
    }
}

glib::wrapper! {
    /// This works as a GObject-compatible class for the MQTTySubscriptionModel model.
    pub struct MQTTySubscriptionMessagesSubscription(ObjectSubclass<imp::MQTTySubscriptionMessagesSubscription>);
}

impl MQTTySubscriptionMessagesSubscription {
    pub(super) fn new(model: &MQTTySubscriptionModel) -> Self {
        let o: Self = glib::Object::builder().build();

        o.set_subscription_model(model);

        o
    }

    pub fn subscription_model(&self) -> MQTTySubscriptionModel {
        self.imp().subscription_model.borrow().clone()
    }

    pub fn connect_changed(&self, cb: impl Fn(&Self) + 'static) -> glib::SignalHandlerId {
        self.connect_closure("changed", false, glib::closure_local!(|o: _| cb(o)))
    }

    pub(super) fn set_subscription_model(&self, model: &MQTTySubscriptionModel) {
        self.imp().subscription_model.replace(model.clone());
        let props_to_notify = ["topic-filter", "qos", "user-subscribed"];
        for prop in props_to_notify {
            self.notify(prop);
        }
        self.emit_by_name::<()>("changed", &[]);
    }
}
