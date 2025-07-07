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
        #[property(get, construct_only)]
        client: OnceCell<MQTTyClient>,

        #[property(get, construct_only)]
        data: OnceCell<MQTTyClientSubscriptionsData>,

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

            let switcher = &self.switcher;
            let spinner = &self.spinner;

            obj.connect_connected_notify(glib::clone!(
                #[weak]
                switcher,
                #[weak]
                spinner,
                move |obj| {
                    // We query the focus before setting its sensitive prop to false
                    let switcher_has_focus = switcher.has_focus();

                    switcher.set_sensitive(false);
                    spinner.set_opacity(1.0);

                    if switcher_has_focus {
                        obj.grab_focus();
                    }

                    let connected = obj.connected();
                    let client = obj.client();
                    let data = obj.data();
                    let subs = data.subscriptions();

                    glib::spawn_future_local(glib::clone!(
                        #[weak]
                        obj,
                        async move {
                            async move {
                                if !connected {
                                    let _ = client.disconnect_client().await;
                                    obj.set_indicator_state("disabled");
                                    obj.set_tooltip_text(None);
                                    return;
                                }

                                futures::future::join_all(
                                    subs.iter()
                                        .filter(|sub| sub.subscribed)
                                        .map(|sub| client.subscribe(&sub.topic_filter, sub.qos)),
                                )
                                .await
                                .into_iter()
                                .filter(|res| res.is_err())
                                .for_each(|res| {
                                    println!("Error while subscribing: {}", res.err().unwrap())
                                });

                                match client.connect_client().await {
                                    Ok(_) => {
                                        obj.set_indicator_state("success");
                                        obj.set_tooltip_text(None);
                                    }
                                    Err(e) => {
                                        obj.set_indicator_state("error");
                                        obj.set_tooltip_text(Some(&format!(
                                            "There was an error while connecting: {}",
                                            e
                                        )));
                                    }
                                };
                            }
                            .await;

                            spinner.set_opacity(0.0);
                            switcher.set_sensitive(true);
                        }
                    ));
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

impl MQTTySubscriptionsConnectionRow {
    pub fn connect_delete_request(&self, cb: impl Fn(&Self) + 'static) -> glib::SignalHandlerId {
        self.connect_closure(
            "delete-request",
            false,
            glib::closure_local!(|o: &Self| cb(o)),
        )
    }

    pub fn connect_edit_request(&self, cb: impl Fn(&Self) + 'static) -> glib::SignalHandlerId {
        self.connect_closure(
            "edit-request",
            false,
            glib::closure_local!(|o: &Self| cb(o)),
        )
    }
}

impl From<&MQTTyClientSubscriptionsData> for MQTTySubscriptionsConnectionRow {
    fn from(value: &MQTTyClientSubscriptionsData) -> Self {
        let conn = value.connection();

        let client = MQTTyClient::builder()
            .clean_start(conn.clean_start)
            .client_id(&conn.client_id)
            .url(&conn.url);

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

        glib::Object::builder()
            .property("client", &client)
            .property("title", &conn.name)
            .property("subtitle", &conn.url)
            .property("connected", conn.connected)
            .property("indicator_state", "disabled")
            .property("data", value)
            .build()
    }
}
