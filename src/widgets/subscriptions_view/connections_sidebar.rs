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
use std::sync::LazyLock;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib::subclass::Signal;
use gtk::{gio, glib};

use crate::application::MQTTyApplication;
use crate::client::MQTTyClientSubscriptionsData;
use crate::persistence::MQTTyMessageStore;
use crate::widgets::{MQTTySubscriptionsConnectionDialog, MQTTySubscriptionsConnectionRow};

use super::handle_gesture_claim_event;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/connections_sidebar.ui")]
    #[properties(wrapper_type = super::MQTTySubscriptionsConnectionsSidebar)]
    pub struct MQTTySubscriptionsConnectionsSidebar {
        #[property(get, construct_only)]
        model: OnceCell<gio::ListStore>,

        #[template_child]
        pub list_box: TemplateChild<gtk::ListBox>,

        #[template_child]
        header_bar: TemplateChild<adw::HeaderBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionsConnectionsSidebar {
        const NAME: &'static str = "MQTTySubscriptionsConnectionsSidebar";

        type Type = super::MQTTySubscriptionsConnectionsSidebar;

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
    impl ObjectImpl for MQTTySubscriptionsConnectionsSidebar {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let model = obj.model();

            self.list_box.bind_model(
                Some(&model),
                glib::clone!(
                    #[weak(rename_to = this)]
                    self,
                    #[upgrade_or_panic]
                    move |o| {
                        let data = o.downcast_ref::<MQTTyClientSubscriptionsData>().unwrap();

                        this.new_connection_row_with_signals(data).upcast()
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

        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
                vec![Signal::builder("connection-activated")
                    .param_types([
                        MQTTySubscriptionsConnectionRow::static_type(),
                        // true when updated or deleted, prefer to use `changed` as a variable name
                        // when connecting to this signal
                        //
                        // Use this mechanism in subscriptions_view.rs to collapse the nav_split_view
                        // when the row is the selected one and it gets updated or deleted.
                        bool::static_type(),
                    ])
                    .build()]
            });
            &*SIGNALS
        }
    }
    impl WidgetImpl for MQTTySubscriptionsConnectionsSidebar {}
    impl NavigationPageImpl for MQTTySubscriptionsConnectionsSidebar {}

    impl MQTTySubscriptionsConnectionsSidebar {
        pub fn new_connection_row_with_signals(
            &self,
            data: &MQTTyClientSubscriptionsData,
        ) -> MQTTySubscriptionsConnectionRow {
            let obj = self.obj();
            let row = MQTTySubscriptionsConnectionRow::from(data);
            // TODO: Store the incoming message in the persistence store, and push it to the
            // :model prop of the corresponding overview (if it exists in the hash map overview).
            let store = MQTTyMessageStore::new(data.connection());
            let client = row.client();
            client.connect_message(|_, message| {});
            row.connect_delete_request(glib::clone!(
                #[weak]
                client,
                #[weak]
                obj,
                move |row| {
                    let index = row.index();

                    obj.emit_by_name::<()>("connection-activated", &[row, &true]);

                    glib::spawn_future_local(async move {
                        let _ = client.disconnect_client().await;

                        obj.model().remove(index as u32);
                    });
                }
            ));
            row.connect_edit_request(glib::clone!(
                #[weak]
                client,
                #[weak]
                obj,
                move |row| {
                    glib::spawn_future_local(glib::clone!(
                        #[weak]
                        row,
                        async move {
                            let data = row.data();
                            let dialog =
                                MQTTySubscriptionsConnectionDialog::new_edit(&data.connection());

                            let app = MQTTyApplication::get_singleton();
                            let window = app.active_window().unwrap();
                            let Some(conn) = dialog.choose_future(&window).await else {
                                return;
                            };

                            obj.emit_by_name::<()>("connection-activated", &[&row, &true]);

                            let new_data = MQTTyClientSubscriptionsData::new();
                            new_data.set_connection(&conn);
                            new_data.set_subscriptions(&data.subscriptions());

                            let _ = client.disconnect_client().await;

                            obj.model().splice(row.index() as u32, 1, &[new_data]);
                        }
                    ));
                }
            ));
            row
        }
    }

    #[gtk::template_callbacks]
    impl MQTTySubscriptionsConnectionsSidebar {
        #[template_callback]
        fn on_connection_activated(&self, row: &MQTTySubscriptionsConnectionRow) {
            let obj = self.obj();
            let list_box = &self.list_box;

            let is_selected = row.is_selected();

            // We are assuming that objects connected to this signal "should"
            // always call unselect_all() method when the row is selected, otherwise
            // this won't behave as expected.
            obj.emit_by_name::<()>("connection-activated", &[row, &false]);

            if !is_selected {
                obj.unselect_all();
                row.set_selectable(true);
                list_box.select_row(Some(row));
            }
        }
    }
}

glib::wrapper! {
    pub struct MQTTySubscriptionsConnectionsSidebar(ObjectSubclass<imp::MQTTySubscriptionsConnectionsSidebar>)
        @extends gtk::Widget, adw::NavigationPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTySubscriptionsConnectionsSidebar {
    pub fn new(model: &gio::ListStore) -> Self {
        glib::Object::builder().property("model", model).build()
    }

    pub fn connect_connection_activated(
        &self,
        cb: impl Fn(&Self, &MQTTySubscriptionsConnectionRow, bool) + 'static,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "connection-activated",
            false,
            glib::closure_local!(|o: &Self,
                                  row: &MQTTySubscriptionsConnectionRow,
                                  changed: bool| cb(
                o, row, changed
            )),
        )
    }

    pub fn unselect_all(&self) {
        let list_box = &self.imp().list_box;
        if let Some(selected) = list_box.selected_row() {
            list_box.unselect_all();
            selected.set_selectable(false);
        }
    }
}
