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

use std::cell::{Cell, OnceCell, RefCell};
use std::sync::LazyLock;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::Signal;
use gtk::glib;

use crate::client::{MQTTyClient, MQTTyClientSubscriptionsData};

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/connection_row.ui")]
    #[properties(wrapper_type = super::MQTTySubscriptionsConnectionRow)]
    pub struct MQTTySubscriptionsConnectionRow {
        #[property(get, set)]
        client: OnceCell<MQTTyClient>,

        #[property(get, set)]
        indicator_state: RefCell<String>,

        #[property(get, set)]
        connected: Cell<bool>,

        #[template_child]
        indicator: TemplateChild<gtk::Box>,
        #[template_child]
        switcher: TemplateChild<gtk::Switch>,
        #[template_child]
        spinner: TemplateChild<adw::Spinner>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionsConnectionRow {
        const NAME: &'static str = "MQTTySubscriptionsConnectionRow";

        type Type = super::MQTTySubscriptionsConnectionRow;

        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.install_action("connection-row.edit", None, |this, _, _| {
                this.emit_by_name::<()>("edit-request", &[]);
            });

            klass.install_action("connection-row.delete", None, |this, _, _| {
                this.emit_by_name::<()>("delete-request", &[]);
            });

            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTySubscriptionsConnectionRow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let indicator = &self.indicator;

            let last_indicator_state: RefCell<Option<String>> = Default::default();

            obj.connect_indicator_state_notify(glib::clone!(
                #[weak]
                indicator,
                move |obj| {
                    if let Some(ref last_state) = last_indicator_state.take() {
                        indicator.remove_css_class(last_state);
                    }
                    let current_state = obj.indicator_state();
                    indicator.add_css_class(&current_state);
                    *last_indicator_state.borrow_mut() = Some(current_state);
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

    impl WidgetImpl for MQTTySubscriptionsConnectionRow {}
    impl PreferencesRowImpl for MQTTySubscriptionsConnectionRow {}
    impl ActionRowImpl for MQTTySubscriptionsConnectionRow {}
    impl ListBoxRowImpl for MQTTySubscriptionsConnectionRow {}
}

glib::wrapper! {
    pub struct MQTTySubscriptionsConnectionRow(ObjectSubclass<imp::MQTTySubscriptionsConnectionRow>)
        @extends adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget, adw::ActionRow,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Actionable;
}

impl From<MQTTyClientSubscriptionsData> for MQTTySubscriptionsConnectionRow {
    fn from(value: MQTTyClientSubscriptionsData) -> Self {
        let conn = value.connection();
        let subs = value.subscriptions();

        let client = MQTTyClient::builder()
            .clean_start(conn.clean_start)
            .url(&conn.url);

        let client = if let Some(client_id) = conn.client_id {
            client.client_id(&client_id)
        } else {
            client
        };

        let client = if let Some(username) = conn.username {
            client.username(&username)
        } else {
            client
        };

        let client = if let Some(password) = conn.password {
            client.password(&password)
        } else {
            client
        };

        let client = client.build();

        let row: Self = glib::Object::builder()
            .property("client", &client)
            .property("title", &conn.name)
            .property("subtitle", &conn.url)
            .property("connected", conn.connected)
            .property("indicator_state", "disabled")
            .build();

        glib::spawn_future_local(glib::clone!(
            #[weak]
            client,
            #[weak]
            row,
            async move {
                futures::future::join_all(
                    subs.iter()
                        .filter(|sub| sub.subscribed)
                        .map(|sub| client.subscribe(&sub.topic_filter, sub.qos)),
                )
                .await
                .into_iter()
                .filter(|res| res.is_err())
                .for_each(|res| println!("Error while subscribing: {}", res.err().unwrap()));

                if conn.connected {
                    client.connect_client().await.unwrap();
                    row.set_indicator_state("success");
                }
            }
        ));

        row
    }
}
