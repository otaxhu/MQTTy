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
use gettextrs::gettext;
use gtk::{gio, glib, pango};

use crate::client::{MQTTyClientQos, MQTTyClientSubscription};
use crate::utils;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/subscription_dialog.ui")]
    #[properties(wrapper_type = super::MQTTySubscriptionDialog)]
    pub struct MQTTySubscriptionDialog {
        #[property(name = "topic", get, set, type = String, member = topic_filter)]
        #[property(name = "qos", get, set, type = MQTTyClientQos, member = qos, builder(Default::default()))]
        #[property(name = "subscribed", get, set, type = bool, member = subscribed)]
        pub subscription: RefCell<MQTTyClientSubscription>,

        #[property(get, set)]
        is_valid: Cell<bool>,

        #[property(get, construct_only)]
        is_new: Cell<bool>,

        #[template_child]
        topic_row: TemplateChild<adw::EntryRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionDialog {
        const NAME: &'static str = "MQTTySubscriptionDialog";

        type Type = super::MQTTySubscriptionDialog;

        type ParentType = adw::AlertDialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTySubscriptionDialog {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            if obj.is_new() {
                obj.set_response_label("save", &gettext("Create"));
            }

            let save_enabled = gtk::ClosureExpression::new::<bool>(
                [obj.property_expression_weak("topic")],
                glib::closure!(|_: Option<glib::Object>, topic: &str| {
                    // TODO: Do further validation
                    !topic.is_empty()
                }),
            );

            save_enabled.bind(&*obj, "is_valid", glib::Object::NONE);

            obj.connect_is_valid_notify(|obj| {
                obj.set_response_enabled("save", obj.is_valid());
            });

            let group = gio::SimpleActionGroup::new();

            utils::connect_qos_action(&*obj, &group);

            obj.insert_action_group("subscription-dialog", Some(&group));

            adw::StyleManager::default().connect_accent_color_notify(glib::clone!(
                #[weak(rename_to = obj)]
                self,
                move |_| {
                    obj.update_topic_row_attrs();
                }
            ));

            obj.connect_topic_notify(glib::clone!(
                #[weak(rename_to = obj)]
                self,
                move |_| {
                    obj.update_topic_row_attrs();
                }
            ));
        }
    }
    impl WidgetImpl for MQTTySubscriptionDialog {}
    impl AdwDialogImpl for MQTTySubscriptionDialog {}
    impl AdwAlertDialogImpl for MQTTySubscriptionDialog {}

    impl MQTTySubscriptionDialog {
        fn update_topic_row_attrs(&self) {
            let obj = self.obj();
            let mut attributes = String::new();
            let topic = obj.topic();
            for (i, _) in topic.match_indices(|c| c == '+' || c == '#') {
                if !attributes.is_empty() {
                    attributes += ",";
                }
                attributes += &format!(
                    "{start} {end} weight bold,{start} {end} foreground {}",
                    utils::get_accent_color_as_hex(),
                    start = i,
                    end = i + 1,
                );
            }
            let parsed_attrs = pango::AttrList::from_string(&attributes).ok();
            self.topic_row.set_attributes(parsed_attrs.as_ref());
        }
    }
}

glib::wrapper! {
    pub struct MQTTySubscriptionDialog(ObjectSubclass<imp::MQTTySubscriptionDialog>)
        @extends adw::Dialog, gtk::Widget, adw::AlertDialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTySubscriptionDialog {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("is_new", true)
            .property("heading", gettext("Create subscription"))
            .build()
    }

    pub fn new_edit(sub: &MQTTyClientSubscription) -> Self {
        glib::Object::builder()
            .property("heading", gettext("Edit subscription"))
            .property("topic", &sub.topic_filter)
            .property("qos", sub.qos)
            .property("subscribed", sub.subscribed)
            .build()
    }

    pub async fn choose_future(
        self,
        parent: &impl IsA<gtk::Widget>,
    ) -> Option<MQTTyClientSubscription> {
        match AlertDialogExtManual::choose_future(self.clone(), parent)
            .await
            .as_ref()
        {
            "save" => Some(self.imp().subscription.take()),
            _ => None,
        }
    }
}
