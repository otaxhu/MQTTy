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

mod connection_dialog;
mod connection_row;
mod connections_sidebar;
mod subscription_dialog;
mod subscription_messages;
mod subscription_row;
mod subscriptions_overview;

pub use connection_dialog::MQTTySubscriptionsConnectionDialog;
pub use connection_row::MQTTySubscriptionsConnectionRow;
pub use connections_sidebar::MQTTySubscriptionsConnectionsSidebar;
pub use subscription_dialog::MQTTySubscriptionDialog;
pub use subscription_messages::MQTTySubscriptionMessages;
pub use subscription_row::MQTTySubscriptionRow;
pub use subscriptions_overview::MQTTySubscriptionsOverview;

use std::cell::{Cell, OnceCell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::application::MQTTyApplication;
use crate::client::MQTTyClientSubscriptionsData;
use crate::display_mode::{MQTTyDisplayMode, MQTTyDisplayModeIface, MQTTyDisplayModeIfaceImpl};

fn handle_gesture_claim_event(ev: &gtk::GestureSingle, picked: &gtk::Widget) {
    // For now we are handling GtkButton s, maybe in the future subscriptions-view gets
    // another widgets that needs to be added here, though any subclass of GtkButton will
    // automatically be handled by this code :p

    let is_button =
        picked.is::<gtk::Button>() || picked.ancestor(gtk::Button::static_type()).is_some();

    if !is_button || ev.current_button() == 3 || ev.downcast_ref::<gtk::GestureDrag>().is_some() {
        ev.set_state(gtk::EventSequenceState::Claimed);
    }
}

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/subscriptions_view.ui")]
    #[properties(wrapper_type = super::MQTTySubscriptionsView)]
    pub struct MQTTySubscriptionsView {
        model: OnceCell<gio::ListStore>,

        data_overview_map:
            RefCell<HashMap<MQTTyClientSubscriptionsData, MQTTySubscriptionsOverview>>,

        #[property(get, set, builder(Default::default()))]
        display_mode: Cell<MQTTyDisplayMode>,

        #[template_child]
        nav_split_view: TemplateChild<adw::NavigationSplitView>,

        #[template_child]
        stack: TemplateChild<gtk::Stack>,

        #[template_child]
        header_bar: TemplateChild<adw::HeaderBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionsView {
        const NAME: &'static str = "MQTTySubscriptionsView";

        type Type = super::MQTTySubscriptionsView;

        type ParentType = adw::Bin;

        type Interfaces = (MQTTyDisplayModeIface,);

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTySubscriptionsView {
        fn constructed(&self) {
            self.parent_constructed();

            let model = self.model();

            let stack = &self.stack;
            let header_bar = &self.header_bar;

            model.connect_notify_local(
                Some("n-items"),
                glib::clone!(
                    #[weak]
                    stack,
                    #[weak]
                    header_bar,
                    move |list, _| {
                        let n_items = list.n_items();

                        stack.set_visible_child_name(if n_items != 0 {
                            "connections"
                        } else {
                            "no-connections"
                        });

                        header_bar.set_show_title(n_items != 0);
                    }
                ),
            );

            let obj = self.obj();

            let nav_split_view = &self.nav_split_view;

            let sidebar = MQTTySubscriptionsConnectionsSidebar::new(model);

            sidebar.connect_connection_activated(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |_, row, changed| {
                    if changed {
                        if row.is_selected() {
                            this.nav_split_view.set_show_content(false);
                        }

                        let _ = this.data_overview_map.borrow_mut().remove(&row.data());
                        return;
                    }
                    if row.is_selected() {
                        this.nav_split_view.set_show_content(false);
                        return;
                    }
                    let nav_split_view = &this.nav_split_view;
                    let mut map = this.data_overview_map.borrow_mut();
                    let overview = map
                        .entry(row.data())
                        .or_insert_with_key(|data| MQTTySubscriptionsOverview::from(data));

                    nav_split_view.set_content(Some(overview));
                    nav_split_view.set_show_content(true);
                }
            ));

            nav_split_view.set_sidebar(Some(&sidebar));

            gtk::ClosureExpression::new::<bool>(
                [
                    obj.property_expression_weak("display_mode"),
                    nav_split_view.property_expression_weak("show-content"),
                ],
                glib::closure!(|_: Option<glib::Object>,
                                display_mode: MQTTyDisplayMode,
                                show_content: bool| {
                    !show_content || display_mode == MQTTyDisplayMode::Mobile
                }),
            )
            .bind(&**nav_split_view, "collapsed", glib::Object::NONE);

            nav_split_view.connect_show_content_notify(move |nav_split_view| {
                if !nav_split_view.shows_content() {
                    sidebar.unselect_all();
                }
            });

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

            header_bar.add_controller(click);
            header_bar.add_controller(drag);
        }
    }
    impl WidgetImpl for MQTTySubscriptionsView {}
    impl BinImpl for MQTTySubscriptionsView {}

    impl MQTTySubscriptionsView {
        pub fn model(&self) -> &gio::ListStore {
            self.model
                .get_or_init(|| gio::ListStore::new::<MQTTyClientSubscriptionsData>())
        }
    }

    impl MQTTyDisplayModeIfaceImpl for MQTTySubscriptionsView {}
}

glib::wrapper! {
    pub struct MQTTySubscriptionsView(ObjectSubclass<imp::MQTTySubscriptionsView>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTySubscriptionsView {
    pub async fn new_connection(&self) {
        let dialog = MQTTySubscriptionsConnectionDialog::new();
        let app = MQTTyApplication::get_singleton();
        let window = app.active_window().unwrap();
        let Some(conn) = dialog.choose_future(&window).await else {
            return;
        };

        let data = MQTTyClientSubscriptionsData::new();

        data.set_connection(&conn);

        let model = self.imp().model();

        model.append(&data);
    }

    // pub fn set_entries(&self, entries: &[MQTTyClient]) {
    //     let model = self.imp().model();
    //     model.splice(0, model.n_items(), entries);
    // }
}
