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

use std::cell::OnceCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use crate::application::MQTTyApplication;
use crate::client::MQTTyClientQos;
use crate::services::subscription_messages::{
    MQTTySubscriptionMessagesClientWrapper, MQTTySubscriptionMessagesSubscription,
};
use crate::utils;
use crate::widgets::MQTTySubscriptionDialog;

use super::toasts;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/subscription_row.ui")]
    #[properties(wrapper_type = super::MQTTySubscriptionRow)]
    pub struct MQTTySubscriptionRow {
        #[property(get, construct_only)]
        client: OnceCell<MQTTySubscriptionMessagesClientWrapper>,

        #[property(get, construct_only)]
        subscription: OnceCell<MQTTySubscriptionMessagesSubscription>,

        #[template_child]
        subscribed_switch: TemplateChild<gtk::Switch>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionRow {
        const NAME: &'static str = "MQTTySubscriptionRow";

        type Type = super::MQTTySubscriptionRow;

        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.install_action("subscription-row.edit", None, |this, _, _| {
                let client = this.client();
                let sub = this.subscription();

                let im = this.imp();
                let subscribed_switch = &im.subscribed_switch;

                glib::spawn_future_local(glib::clone!(
                    #[weak]
                    subscribed_switch,
                    async move {
                        let app = MQTTyApplication::get_singleton();
                        let window = app.active_window().unwrap();

                        let dialog = MQTTySubscriptionDialog::new_edit(&sub.subscription_model());

                        let Some(new_model) = dialog.choose_future(&window).await else {
                            return;
                        };

                        subscribed_switch.set_sensitive(false);

                        match client.update_subscription(&sub, &new_model).await {
                            (false, _) => {
                                toasts::subscription_already_exists();
                            }
                            // Updated succesfully
                            (true, None) => {}
                            _ => {}
                        }

                        subscribed_switch.set_sensitive(true);
                    }
                ));
            });

            klass.install_action("subscription-row.delete", None, |this, _, _| {
                let client = this.client();
                let sub = this.subscription();

                glib::spawn_future_local(async move {
                    // TODO: Show an AlertDialog to confirm deletion

                    let _ = client.remove_subscription(&sub).await;
                });
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

            let subscribed_switch = &self.subscribed_switch;

            let sub = obj.subscription();
            let client = obj.client();

            sub.bind_property("user-subscribed", &**subscribed_switch, "active")
                .sync_create()
                .bidirectional()
                .build();

            sub.bind_property("qos", &*obj, "subtitle")
                .sync_create()
                .transform_to(|_, qos: MQTTyClientQos| Some(qos.translated()))
                .build();

            sub.connect_user_subscribed_notify(glib::clone!(
                #[weak]
                subscribed_switch,
                #[weak]
                client,
                #[weak]
                obj,
                move |sub| {
                    let subscribed_switch_has_focus = subscribed_switch.has_focus();

                    subscribed_switch.set_sensitive(false);

                    if subscribed_switch_has_focus {
                        obj.grab_focus();
                    }

                    glib::spawn_future_local(glib::clone!(
                        #[weak]
                        sub,
                        async move {
                            let _ = client.sync_subscription(&sub).await;

                            subscribed_switch.set_sensitive(true);
                        }
                    ));
                }
            ));

            sub.connect_topic_filter_notify(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |_| {
                    this.update_title_pango();
                }
            ));

            let man = adw::StyleManager::default();

            man.connect_accent_color_notify(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |_| {
                    this.update_title_pango();
                }
            ));

            man.connect_dark_notify(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |_| {
                    this.update_title_pango();
                }
            ));

            self.update_title_pango();
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

            let escaped_topic = glib::markup_escape_text(&obj.subscription().topic_filter());

            let open_pango_tag = format!(
                "<span foreground='{}' weight='bold'>",
                utils::get_fg_accent_color_as_hex()
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

impl MQTTySubscriptionRow {
    pub fn new(
        sub: &MQTTySubscriptionMessagesSubscription,
        client: &MQTTySubscriptionMessagesClientWrapper,
    ) -> Self {
        glib::Object::builder()
            .property("subscription", sub)
            .property("client", client)
            .build()
    }
}
