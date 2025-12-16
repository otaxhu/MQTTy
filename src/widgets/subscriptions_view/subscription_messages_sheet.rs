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
use gtk::{gio, glib};

use crate::client::MQTTyClientMessage;
use crate::services::subscription_messages::MQTTySubscriptionMessagesClientWrapper;
use crate::utils;

use super::handle_gesture_claim_event;

mod imp {

    use crate::widgets::subscriptions_view::MQTTySubscriptionsMessageRow;

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(
        resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/subscription_messages_sheet.ui"
    )]
    #[properties(wrapper_type = super::MQTTySubscriptionMessagesSheet)]
    pub struct MQTTySubscriptionMessagesSheet {
        model: OnceCell<gio::ListStore>,

        msg_cursor_id: Cell<Option<i64>>,

        #[property(get, construct_only)]
        client: OnceCell<MQTTySubscriptionMessagesClientWrapper>,

        #[template_child]
        stack: TemplateChild<gtk::Stack>,

        #[template_child]
        list_box: TemplateChild<gtk::ListBox>,

        #[template_child]
        load_messages_button: TemplateChild<gtk::Button>,

        #[template_child]
        header_bar: TemplateChild<adw::HeaderBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionMessagesSheet {
        const NAME: &'static str = "MQTTySubscriptionMessagesSheet";

        type Type = super::MQTTySubscriptionMessagesSheet;

        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTySubscriptionMessagesSheet {
        fn constructed(&self) {
            self.parent_constructed();

            let stack = &self.stack;
            let list_box = &self.list_box;

            let model = self.model();
            model.connect_notify_local(
                Some("n-items"),
                glib::clone!(
                    #[weak]
                    stack,
                    move |model, _| {
                        stack.set_visible_child_name(if model.n_items() != 0 {
                            "messages"
                        } else {
                            "no-messages"
                        });
                    }
                ),
            );

            list_box.bind_model(Some(model), |o| {
                let message = o.downcast_ref::<MQTTyClientMessage>().unwrap();

                MQTTySubscriptionsMessageRow::new(message).upcast()
            });

            // First call, to setup the first messages.
            self.on_load_messages();

            let obj = self.obj();
            let client = obj.client();

            client.connect_message(glib::clone!(
                #[weak]
                model,
                move |_, msg| {
                    model.insert(0, &msg);
                }
            ));

            // Reset list if client's identity fields changes.

            client.connect_identity_changed(glib::clone!(
                #[weak]
                model,
                #[weak(rename_to = this)]
                self,
                move |_| {
                    model.remove_all();
                    this.msg_cursor_id.set(None);
                    this.on_load_messages();
                }
            ));

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
                let (x, y) = drag.start_point().unwrap();
                let offset_point = (off_x, off_y);
                let picked = drag
                    .widget()
                    .unwrap()
                    .pick(x, y, gtk::PickFlags::DEFAULT)
                    .unwrap();

                if utils::gtk_drag_check_threshold_double(&picked, (0.0, 0.0), offset_point) {
                    handle_gesture_claim_event(drag.upcast_ref(), &picked);
                }
            });

            self.header_bar.add_controller(click);
            self.header_bar.add_controller(drag);
        }
    }
    impl WidgetImpl for MQTTySubscriptionMessagesSheet {}
    impl BinImpl for MQTTySubscriptionMessagesSheet {}

    impl MQTTySubscriptionMessagesSheet {
        pub fn model(&self) -> &gio::ListStore {
            self.model
                .get_or_init(|| gio::ListStore::new::<MQTTyClientMessage>())
        }
    }

    #[gtk::template_callbacks]
    impl MQTTySubscriptionMessagesSheet {
        #[template_callback]
        fn on_load_messages(&self) {
            let obj = self.obj();

            let cursor_id = self.msg_cursor_id.get();
            let model = self.model();
            let client = obj.client();

            let load_messages_button = &self.load_messages_button;

            let limit = 25;

            let (msg, new_cursor_id) = match client.get_recent_messages(cursor_id, limit) {
                Ok(msg) => msg,
                Err(e) => {
                    println!("Error while getting the messages: {e:?}");
                    (vec![], None)
                }
            };

            self.msg_cursor_id.set(new_cursor_id);
            model.splice(model.n_items(), 0, &msg);
            load_messages_button.set_visible(new_cursor_id.is_some());
        }
    }
}

glib::wrapper! {
    pub struct MQTTySubscriptionMessagesSheet(ObjectSubclass<imp::MQTTySubscriptionMessagesSheet>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTySubscriptionMessagesSheet {
    pub fn new(client: &MQTTySubscriptionMessagesClientWrapper) -> Self {
        glib::Object::builder().property("client", client).build()
    }
}
