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

use std::cell::{Cell, OnceCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use formatx::formatx;
use gettextrs::gettext;
use gtk::glib;

use crate::client::MQTTyClientConnectionState;
use crate::services::subscription_messages::MQTTySubscriptionMessagesClientWrapper;
use crate::widgets::AdwIndicatorBin;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/connection_row.ui")]
    #[properties(wrapper_type = super::MQTTySubscriptionsConnectionRow)]
    pub struct MQTTySubscriptionsConnectionRow {
        #[property(get, construct_only)]
        client: OnceCell<MQTTySubscriptionMessagesClientWrapper>,

        #[property(get, set)]
        n_unread: Cell<u32>,

        #[template_child]
        switcher: TemplateChild<gtk::Switch>,
        #[template_child]
        spinner: TemplateChild<adw::Spinner>,
        #[template_child]
        indicator_bin: TemplateChild<AdwIndicatorBin>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionsConnectionRow {
        const NAME: &'static str = "MQTTySubscriptionsConnectionRow";

        type Type = super::MQTTySubscriptionsConnectionRow;

        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.install_action("connection-row.edit", None, |this, _, _| {
                let Ok(index) = u32::try_from(this.index()) else {
                    return;
                };

                this.activate_action(
                    "subscriptions-view.edit-connection",
                    Some(&index.to_variant()),
                )
                .unwrap();
            });

            klass.install_action("connection-row.delete", None, |this, _, _| {
                let Ok(index) = u32::try_from(this.index()) else {
                    return;
                };

                this.activate_action(
                    "subscriptions-view.delete-connection",
                    Some(&index.to_variant()),
                )
                .unwrap();
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
            let client = obj.client();

            let switcher = &self.switcher;
            let spinner = &self.spinner;
            let indicator_bin = &self.indicator_bin;

            client
                .bind_property("name", &*obj, "title")
                .sync_create()
                .build();
            client
                .bind_property("url", &*obj, "subtitle")
                .sync_create()
                .build();

            client
                .bind_property("user-connected", &**switcher, "active")
                .sync_create()
                .bidirectional()
                .build();

            client.connect_user_connected_notify(glib::clone!(
                #[weak]
                spinner,
                #[weak]
                switcher,
                #[weak]
                obj,
                move |client| {
                    switcher.remove_css_class("error");
                    obj.set_tooltip_text(None);

                    let switcher_has_focus = switcher.has_focus();

                    switcher.set_sensitive(false);

                    if switcher_has_focus {
                        obj.grab_focus();
                    }

                    spinner.set_visible(true);

                    glib::spawn_future_local(glib::clone!(
                        #[weak]
                        client,
                        async move {
                            match client.sync_user_connected().await {
                                Err(e) => {
                                    obj.set_tooltip_text(Some(
                                        formatx!(gettext("There was an error: {}"), e)
                                            .unwrap()
                                            .as_str(),
                                    ));

                                    switcher.add_css_class("error");
                                }
                                // Ok(...) branch code is handled in connection_state_changed
                                // handler below.
                                //
                                // It is guaranteed that it will be called with Connected state.
                                _ => {}
                            }
                            spinner.set_visible(false);
                            switcher.set_sensitive(true);
                        }
                    ));
                }
            ));

            client.connect_connection_state_changed(glib::clone!(
                #[weak]
                switcher,
                #[weak]
                spinner,
                #[weak]
                obj,
                move |client, state| {
                    let user_connected = client.user_connected();
                    let (text, is_err, spinning) = match (state, user_connected) {
                        (MQTTyClientConnectionState::Connected, _)
                        | (MQTTyClientConnectionState::Disconnected, false) => (None, false, false),
                        (MQTTyClientConnectionState::Disconnected, true) => (
                            Some(gettext("Client got disconnected by broker")),
                            true,
                            false,
                        ),
                        (MQTTyClientConnectionState::Reconnecting, _) => {
                            (Some(gettext("Reconnecting...")), false, true)
                        }
                        (MQTTyClientConnectionState::ReconnectFailure, _) => {
                            (Some(gettext("Reconnection failed")), true, false)
                        }
                        (MQTTyClientConnectionState::SessionTakenOver, _) => (
                            Some(gettext(
                                "Another client took over the session (duplicated client ID)",
                            )),
                            true,
                            false,
                        ),
                    };
                    obj.set_tooltip_text(text.as_ref().map(|s| s.as_str()));
                    if is_err {
                        switcher.add_css_class("error");
                    } else {
                        switcher.remove_css_class("error");
                    }
                    spinner.set_visible(spinning);
                }
            ));

            obj.connect_n_unread_notify(glib::clone!(
                #[weak]
                indicator_bin,
                move |obj| {
                    indicator_bin.set_needs_attention(obj.n_unread() > 0);
                    indicator_bin.set_badge_number(obj.n_unread());
                }
            ));
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
    pub fn new(client: &MQTTySubscriptionMessagesClientWrapper) -> Self {
        glib::Object::builder().property("client", client).build()
    }
}
