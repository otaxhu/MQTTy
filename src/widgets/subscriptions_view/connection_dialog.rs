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
use gtk::glib;

use crate::client::MQTTyClientConnection;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/connection_dialog.ui")]
    #[properties(wrapper_type = super::MQTTySubscriptionsConnectionDialog)]
    pub struct MQTTySubscriptionsConnectionDialog {
        #[property(name = "name", get, set, type = String, member = name)]
        #[property(name = "client-id", get, set, type = Option<String>, member = client_id)]
        #[property(name = "url", get, set, type = String, member = url)]
        #[property(name = "username", get, set, type = Option<String>, member = username)]
        #[property(name = "password", get, set, type = Option<String>, member = password)]
        #[property(name = "clean-start", get, set, type = bool, member = clean_start)]
        #[property(name = "connected", get, set, type = bool, member = connected)]
        pub connection: RefCell<MQTTyClientConnection>,

        #[property(get, set)]
        is_valid: Cell<bool>,

        #[property(get, construct_only)]
        is_new: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionsConnectionDialog {
        const NAME: &'static str = "MQTTySubscriptionsConnectionDialog";

        type Type = super::MQTTySubscriptionsConnectionDialog;

        type ParentType = adw::AlertDialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {
            Self {
                connection: RefCell::new(MQTTyClientConnection {
                    client_id: Some("".to_string()),
                    username: Some("".to_string()),
                    password: Some("".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTySubscriptionsConnectionDialog {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            if obj.is_new() {
                obj.set_response_label("save", &gettext("Create"));
            }

            let save_enabled = gtk::ClosureExpression::new::<bool>(
                [
                    obj.property_expression_weak("name"),
                    obj.property_expression_weak("url"),
                ],
                glib::closure!(|_: Option<glib::Object>, name: &str, url: &str| {
                    // TODO: Do further validation
                    !name.is_empty() && !url.is_empty()
                }),
            );

            save_enabled.bind(&*obj, "is_valid", glib::Object::NONE);

            obj.connect_is_valid_notify(|obj| {
                obj.set_response_enabled("save", obj.is_valid());
            });
        }
    }
    impl WidgetImpl for MQTTySubscriptionsConnectionDialog {}
    impl AdwDialogImpl for MQTTySubscriptionsConnectionDialog {}
    impl AdwAlertDialogImpl for MQTTySubscriptionsConnectionDialog {}
}

glib::wrapper! {
    pub struct MQTTySubscriptionsConnectionDialog(ObjectSubclass<imp::MQTTySubscriptionsConnectionDialog>)
        @extends gtk::Widget, adw::Dialog, adw::AlertDialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTySubscriptionsConnectionDialog {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("heading", gettext("New connection"))
            .property("is-new", true)
            .build()
    }

    pub fn new_edit(conn: &MQTTyClientConnection) -> Self {
        glib::Object::builder()
            .property("heading", gettext("Edit connection"))
            .property("name", &conn.name)
            .property("client-id", conn.client_id.as_ref())
            .property("url", &conn.url)
            .property("username", conn.username.as_ref())
            .property("password", conn.password.as_ref())
            .property("clean-start", conn.clean_start)
            .property("connected", conn.connected)
            .build()
    }

    /// Returns an already validated MQTTyClientConnection struct, or None if
    /// "cancel" or "close" were the options selected
    pub async fn choose_future(
        self,
        parent: &impl IsA<gtk::Widget>,
    ) -> Option<MQTTyClientConnection> {
        match AlertDialogExtManual::choose_future(self.clone(), parent)
            .await
            .as_ref()
        {
            "save" => Some(self.imp().connection.take()),
            _ => None,
        }
    }
}
