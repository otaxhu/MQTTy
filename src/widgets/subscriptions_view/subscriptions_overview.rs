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
use crate::services::subscription_messages::{
    MQTTySubscriptionMessagesClientWrapper, MQTTySubscriptionMessagesSubscription,
};
use crate::utils;
use crate::widgets::{
    MQTTySubscriptionDialog, MQTTySubscriptionMessagesSheet, MQTTySubscriptionRow,
};

use super::{handle_gesture_claim_event, toasts};

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(
        resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/subscriptions_overview.ui"
    )]
    #[properties(wrapper_type = super::MQTTySubscriptionsOverview)]
    pub struct MQTTySubscriptionsOverview {
        #[property(get, construct_only)]
        client: OnceCell<MQTTySubscriptionMessagesClientWrapper>,

        #[template_child]
        list_box: TemplateChild<gtk::ListBox>,

        #[template_child]
        stack: TemplateChild<gtk::Stack>,

        #[template_child]
        bottom_sheet: TemplateChild<adw::BottomSheet>,

        #[template_child]
        header_bar: TemplateChild<adw::HeaderBar>,

        #[template_child]
        window_title: TemplateChild<adw::WindowTitle>,

        #[template_child]
        reset_session_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionsOverview {
        const NAME: &'static str = "MQTTySubscriptionsOverview";

        type Type = super::MQTTySubscriptionsOverview;

        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTySubscriptionsOverview {
        fn constructed(&self) {
            let obj = self.obj();

            let client = obj.client();

            let window_title = &self.window_title;
            let stack = &self.stack;
            let list_box = &self.list_box;
            let bottom_sheet = &self.bottom_sheet;

            bottom_sheet.set_sheet(Some(&MQTTySubscriptionMessagesSheet::new(&client)));

            client
                .bind_property("name", &**window_title, "subtitle")
                .sync_create()
                .build();

            let subs_list = client.subscriptions();

            subs_list
                .bind_property("n-items", &**stack, "visible-child-name")
                .sync_create()
                .transform_to(|_, n_items: u32| {
                    Some(if n_items == 0 {
                        "no-subscriptions"
                    } else {
                        "subscriptions"
                    })
                })
                .build();

            list_box.bind_model(
                Some(&subs_list),
                glib::clone!(
                    #[weak]
                    client,
                    #[upgrade_or_panic]
                    move |sub| {
                        let sub = sub
                            .downcast_ref::<MQTTySubscriptionMessagesSubscription>()
                            .unwrap();

                        let row = MQTTySubscriptionRow::new(sub, &client);

                        row.upcast()
                    }
                ),
            );

            let click = gtk::GestureClick::new();
            click.set_button(0);
            click.set_propagation_phase(gtk::PropagationPhase::Capture);
            click.connect_pressed(|click, n_presses, x, y| {
                if n_presses > 1 {
                    click.set_state(gtk::EventSequenceState::Claimed);
                    return;
                }

                let picked = click
                    .widget()
                    .unwrap()
                    .pick(x, y, gtk::PickFlags::DEFAULT)
                    .unwrap();

                handle_gesture_claim_event(click.upcast_ref(), &picked);
            });

            let drag = gtk::GestureDrag::new();
            drag.set_propagation_phase(gtk::PropagationPhase::Capture);
            drag.connect_drag_update(|drag, off_x, off_y| {
                let start_point @ (x, y) = drag.start_point().unwrap();
                let offset_point = (off_x, off_y);
                let picked = drag
                    .widget()
                    .unwrap()
                    .pick(x, y, gtk::PickFlags::DEFAULT)
                    .unwrap();

                if utils::gtk_drag_check_threshold_double(&picked, start_point, offset_point) {
                    handle_gesture_claim_event(drag.upcast_ref(), &picked);
                }
            });

            self.header_bar.add_controller(click);
            self.header_bar.add_controller(drag);
        }
    }

    impl WidgetImpl for MQTTySubscriptionsOverview {}
    impl NavigationPageImpl for MQTTySubscriptionsOverview {}

    #[gtk::template_callbacks]
    impl MQTTySubscriptionsOverview {
        #[template_callback]
        fn on_new_subscription(&self) {
            let obj = self.obj();
            let client = obj.client();

            glib::spawn_future_local(async move {
                let app = MQTTyApplication::get_singleton();
                let window = app.active_window().unwrap();
                let dialog = MQTTySubscriptionDialog::new();

                let Some(sub) = dialog.choose_future(&window).await else {
                    return;
                };

                if client.contains_subscription(&sub.topic_filter) {
                    // Subscription already exists
                    toasts::subscription_already_exists();
                    return;
                }

                let subs_list = client.subscriptions();

                client.add_subscriptions(&[sub]);
                let _ = client
                    .sync_subscription(
                        &subs_list
                            .item(subs_list.n_items() - 1)
                            .unwrap()
                            .downcast::<_>()
                            .unwrap(),
                    )
                    .await;
            });
        }

        #[template_callback]
        fn on_reset_session(&self) {
            let obj = self.obj();
            let client = obj.client();
            let reset_session_button = &self.reset_session_button;

            reset_session_button.set_sensitive(false);

            glib::spawn_future_local(glib::clone!(
                #[weak]
                reset_session_button,
                async move {
                    let _ = client.reset_session().await;
                    reset_session_button.set_sensitive(true);
                }
            ));
        }
    }
}

glib::wrapper! {
    pub struct MQTTySubscriptionsOverview(ObjectSubclass<imp::MQTTySubscriptionsOverview>)
        @extends gtk::Widget, adw::NavigationPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTySubscriptionsOverview {
    pub fn new(client: &MQTTySubscriptionMessagesClientWrapper) -> Self {
        glib::Object::builder().property("client", client).build()
    }
}
