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

use crate::client::MQTTyClientSubscription;
use crate::utils;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/subscription_row.ui")]
    #[properties(wrapper_type = super::MQTTySubscriptionRow)]
    pub struct MQTTySubscriptionRow {
        #[property(get, set)]
        topic: RefCell<String>,

        #[property(get, set)]
        subscribed: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionRow {
        const NAME: &'static str = "MQTTySubscriptionRow";

        type Type = super::MQTTySubscriptionRow;

        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.install_action("subscription-row.edit", None, |this, _, _| {
                this.emit_by_name::<()>("edit-request", &[]);
            });

            klass.install_action("subscription-row.delete", None, |this, _, _| {
                this.emit_by_name::<()>("delete-request", &[]);
            });

            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTySubscriptionRow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            obj.connect_topic_notify(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |_| {
                    this.update_title_pango();
                }
            ));

            adw::StyleManager::default().connect_accent_color_notify(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |_| {
                    this.update_title_pango();
                }
            ));
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
                vec![
                    Signal::builder("edit-request").build(),
                    Signal::builder("delete-request").build(),
                ]
            });
            &*SIGNALS
        }
    }
    impl WidgetImpl for MQTTySubscriptionRow {}
    impl ListBoxRowImpl for MQTTySubscriptionRow {}
    impl PreferencesRowImpl for MQTTySubscriptionRow {}
    impl ActionRowImpl for MQTTySubscriptionRow {}

    impl MQTTySubscriptionRow {
        fn update_title_pango(&self) {
            // We are replacing the wildcard MQTT characters '+' and '#' with a
            // colored and bold version of the same character, we are using Pango markup
            // to accomplish this

            let obj = self.obj();

            let escaped_topic = glib::markup_escape_text(&obj.topic());

            let open_pango_tag = format!(
                "<span foreground='{}' weight='bold'>",
                utils::get_accent_color_as_hex()
            );

            obj.set_title(
                &escaped_topic
                    .replace("#", &format!("{open_pango_tag}#</span>"))
                    .replace("+", &format!("{open_pango_tag}+</span>")),
            );
        }
    }
}

glib::wrapper! {
    pub struct MQTTySubscriptionRow(ObjectSubclass<imp::MQTTySubscriptionRow>)
        @extends gtk::ListBoxRow, gtk::Widget, adw::PreferencesRow, adw::ActionRow,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Actionable;
}

impl From<&MQTTyClientSubscription> for MQTTySubscriptionRow {
    fn from(value: &MQTTyClientSubscription) -> Self {
        glib::Object::builder()
            .property("topic", &value.topic_filter)
            .property("subtitle", value.qos.translated())
            .property("subscribed", value.subscribed)
            .property("use-markup", true)
            .build()
    }
}

impl MQTTySubscriptionRow {
    pub fn connect_edit_request(&self, cb: impl Fn(&Self) + 'static) -> glib::SignalHandlerId {
        self.connect_closure(
            "edit-request",
            false,
            glib::closure_local!(|o: &Self| cb(o)),
        )
    }

    pub fn connect_delete_request(&self, cb: impl Fn(&Self) + 'static) -> glib::SignalHandlerId {
        self.connect_closure(
            "delete-request",
            false,
            glib::closure_local!(|o: &Self| cb(o)),
        )
    }
}
