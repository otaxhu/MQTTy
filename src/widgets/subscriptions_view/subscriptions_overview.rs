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

use std::cell::{OnceCell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use crate::application::MQTTyApplication;
use crate::client::{MQTTyClientSubscription, MQTTyClientSubscriptionsData};
use crate::widgets::{MQTTySubscriptionDialog, MQTTySubscriptionRow};

use super::handle_gesture_claim_event;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(
        resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/subscriptions_overview.ui"
    )]
    #[properties(wrapper_type = super::MQTTySubscriptionsOverview)]
    pub struct MQTTySubscriptionsOverview {
        #[property(get, construct_only)]
        data: OnceCell<MQTTyClientSubscriptionsData>,

        #[property(get, set)]
        subtitle: RefCell<String>,

        #[template_child]
        list_box: TemplateChild<gtk::ListBox>,

        #[template_child]
        stack: TemplateChild<gtk::Stack>,

        #[template_child]
        bottom_sheet: TemplateChild<adw::BottomSheet>,

        #[template_child]
        header_bar: TemplateChild<adw::HeaderBar>,
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
            self.update_list_box();

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
            drag.connect_drag_begin(|drag, x, y| {
                let picked = drag
                    .widget()
                    .unwrap()
                    .pick(x, y, gtk::PickFlags::DEFAULT)
                    .unwrap();

                let signal_id: Rc<RefCell<Option<glib::SignalHandlerId>>> = Default::default();

                *signal_id.borrow_mut() = Some(drag.connect_drag_update(glib::clone!(
                    #[strong]
                    signal_id,
                    move |drag, _x, _y| {
                        drag.disconnect(signal_id.take().unwrap());

                        handle_gesture_claim_event(drag.upcast_ref(), &picked);
                    }
                )));
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
            let data = self.obj().data();

            glib::spawn_future_local(glib::clone!(
                #[weak(rename_to = this)]
                self,
                #[weak]
                data,
                async move {
                    let app = MQTTyApplication::get_singleton();
                    let window = app.active_window().unwrap();
                    let dialog = MQTTySubscriptionDialog::new();

                    let Some(sub) = dialog.choose_future(&window).await else {
                        return;
                    };

                    data.set_subscriptions(
                        data.subscriptions()
                            .into_iter()
                            .chain(std::iter::once(sub))
                            .collect::<Vec<_>>()
                            .as_ref(),
                    );

                    this.update_list_box();
                }
            ));
        }
    }

    impl MQTTySubscriptionsOverview {
        fn update_list_box(&self) {
            let obj = self.obj();
            let subs = obj.data().subscriptions();

            let list_box = &self.list_box;

            list_box.remove_all();
            for sub in &subs {
                list_box.append(&self.new_row_with_signals(sub));
            }

            self.stack.set_visible_child_name(if subs.len() != 0 {
                "subscriptions"
            } else {
                "no-subscriptions"
            });
        }

        fn new_row_with_signals(&self, sub: &MQTTyClientSubscription) -> MQTTySubscriptionRow {
            let obj = self.obj();
            let data = obj.data();
            let row = MQTTySubscriptionRow::from(sub);
            row.connect_subscribed_notify(glib::clone!(
                #[weak]
                data,
                move |row| {
                    let mut subs = data.subscriptions();
                    subs[row.index() as usize].subscribed = row.subscribed();
                    data.set_subscriptions(&subs);

                    // There is no need to call `this.update_list_box()`
                }
            ));
            row.connect_delete_request(glib::clone!(
                #[weak]
                data,
                #[weak(rename_to = this)]
                self,
                move |row| {
                    let mut subs = data.subscriptions();

                    subs.remove(row.index() as usize);

                    data.set_subscriptions(&subs);

                    this.update_list_box();
                }
            ));
            row.connect_edit_request(glib::clone!(
                #[weak]
                data,
                #[weak(rename_to = this)]
                self,
                move |row| {
                    glib::spawn_future_local(glib::clone!(
                        #[weak]
                        row,
                        async move {
                            let mut subs = data.subscriptions();
                            let sub = &subs[row.index() as usize];

                            let app = MQTTyApplication::get_singleton();
                            let window = app.active_window().unwrap();
                            let dialog = MQTTySubscriptionDialog::new_edit(sub);
                            let Some(new_sub) = dialog.choose_future(&window).await else {
                                return;
                            };
                            subs[row.index() as usize] = new_sub;
                            data.set_subscriptions(&subs);
                            this.update_list_box();
                        }
                    ));
                }
            ));
            row
        }
    }
}

glib::wrapper! {
    pub struct MQTTySubscriptionsOverview(ObjectSubclass<imp::MQTTySubscriptionsOverview>)
        @extends gtk::Widget, adw::NavigationPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl From<&MQTTyClientSubscriptionsData> for MQTTySubscriptionsOverview {
    fn from(value: &MQTTyClientSubscriptionsData) -> Self {
        glib::Object::builder()
            .property("subtitle", value.connection().name)
            .property("data", value)
            .build()
    }
}
