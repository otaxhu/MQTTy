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
mod message_row;
mod subscription_dialog;
mod subscription_messages_sheet;
mod subscription_row;
mod subscriptions_overview;
mod toasts;

pub use connection_dialog::MQTTySubscriptionsConnectionDialog;
pub use connection_row::MQTTySubscriptionsConnectionRow;
pub use message_row::MQTTySubscriptionsMessageRow;
pub use subscription_dialog::MQTTySubscriptionDialog;
pub use subscription_messages_sheet::MQTTySubscriptionMessagesSheet;
pub use subscription_row::MQTTySubscriptionRow;
pub use subscriptions_overview::MQTTySubscriptionsOverview;

use std::cell::{Cell, OnceCell, RefCell};
use std::collections::HashMap;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use crate::application::MQTTyApplication;
use crate::display_mode::{MQTTyDisplayMode, MQTTyDisplayModeIface, MQTTyDisplayModeIfaceImpl};
use crate::main_window::MQTTyWindow;
use crate::services::subscription_messages::{
    MQTTySubscriptionMessagesClientWrapper, MQTTySubscriptionMessagesController,
};
use crate::utils;

fn handle_gesture_claim_event(ev: &gtk::GestureSingle, picked: &gtk::Widget) {
    // For now we are handling GtkButton s, maybe in the future subscriptions-view gets
    // another widgets that needs to be added here, though any subclass of GtkButton will
    // automatically be handled by this code :p

    let is_button =
        picked.is::<gtk::Button>() || picked.ancestor(gtk::Button::static_type()).is_some();

    if !is_button || ev.current_button() > 1 || ev.downcast_ref::<gtk::GestureDrag>().is_some() {
        ev.set_state(gtk::EventSequenceState::Claimed);
    }
}

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/subscriptions_view.ui")]
    #[properties(wrapper_type = super::MQTTySubscriptionsView)]
    pub struct MQTTySubscriptionsView {
        controller: OnceCell<MQTTySubscriptionMessagesController>,

        clients_overview_map:
            RefCell<HashMap<MQTTySubscriptionMessagesClientWrapper, MQTTySubscriptionsOverview>>,

        last_selected_row: RefCell<Option<gtk::ListBoxRow>>,

        #[property(get, set, builder(Default::default()))]
        display_mode: Cell<MQTTyDisplayMode>,

        #[template_child]
        nav_split_view: TemplateChild<adw::NavigationSplitView>,

        #[template_child]
        stack: TemplateChild<gtk::Stack>,

        #[template_child]
        sidebar_header_bar: TemplateChild<adw::HeaderBar>,

        #[template_child]
        sidebar: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionsView {
        const NAME: &'static str = "MQTTySubscriptionsView";

        type Type = super::MQTTySubscriptionsView;

        type ParentType = adw::Bin;

        type Interfaces = (MQTTyDisplayModeIface,);

        fn class_init(klass: &mut Self::Class) {
            // Action is called by connection rows when they are deleted.
            klass.install_action(
                "subscriptions-view.delete-connection",
                // u32 index of the client in `controllers.clients` ListModel
                Some(u32::static_variant_type().as_ref()),
                |this, _, index| {
                    let index = index.unwrap().get::<u32>().unwrap();

                    let im = this.imp();
                    let controller = im.controller();

                    let clients = controller.clients();
                    let Some(client) = clients.item(index).map(|c| {
                        c.downcast::<MQTTySubscriptionMessagesClientWrapper>()
                            .unwrap()
                    }) else {
                        return;
                    };

                    glib::spawn_future_local(glib::clone!(
                        #[weak]
                        im,
                        #[weak]
                        controller,
                        async move {
                            // TODO: Show an AlertDialog to confirm the deletion

                            let _ = controller.remove_client(&client).await;

                            // This prevents a weird flickering due to setting
                            // nav_split_view:content to NONE and then again after
                            // in the "::row-selected" signal handler.
                            //
                            // Here we are directly selecting a row, then in
                            // "::row-selected" it gets handled.
                            im.sidebar.select_row(
                                im.sidebar
                                    .row_at_index(
                                        index.min(clients.n_items().saturating_sub(1)) as _
                                    )
                                    .as_ref(),
                            );

                            if let (Some(overview), Some(nav_content)) = (
                                im.clients_overview_map.borrow_mut().remove(&client),
                                im.nav_split_view.content(),
                            ) {
                                if overview == nav_content {
                                    // Above should hold true only if there was 1 client
                                    // (at this point 0 because of the removal).
                                    //
                                    // We drop it so that there is no memory leak.
                                    im.nav_split_view.set_content(adw::NavigationPage::NONE);
                                }
                            }

                            let borrow = im.last_selected_row.borrow();
                            let delete_last = match &*borrow {
                                Some(r) => {
                                    let row = r
                                        .downcast_ref::<MQTTySubscriptionsConnectionRow>()
                                        .unwrap();
                                    row.client() == client
                                }
                                _ => false,
                            };
                            drop(borrow);
                            if delete_last {
                                // We drop it so that there is no memory leak.
                                im.last_selected_row.take();
                            }
                        }
                    ));
                },
            );
            // Action is called by connection rows when they are edited.
            klass.install_action(
                "subscriptions-view.edit-connection",
                Some(u32::static_variant_type().as_ref()),
                |this, _, index| {
                    // u32 index of the client in `controllers.clients` ListModel
                    let index = index.unwrap().get::<u32>().unwrap();

                    let im = this.imp();
                    let controller = im.controller();

                    let clients = controller.clients();
                    let Some(client) = clients.item(index).map(|c| {
                        c.downcast::<MQTTySubscriptionMessagesClientWrapper>()
                            .unwrap()
                    }) else {
                        return;
                    };

                    glib::spawn_future_local(glib::clone!(
                        #[weak]
                        im,
                        async move {
                            let app = MQTTyApplication::get_singleton();
                            let dialog = MQTTySubscriptionsConnectionDialog::new_edit(
                                &client.connection_model(),
                            );
                            let window = app.active_window().unwrap();
                            let Some(conn) = dialog.choose_future(&window).await else {
                                return;
                            };

                            let controller = im.controller();

                            if controller.contains_connection_for_update(
                                &client,
                                &conn.as_ref().url,
                                &conn.client_id,
                            ) {
                                // The connection already exists
                                toasts::client_already_exists();
                                return;
                            }

                            // Client is ok to be updated, BUT we don't call
                            // client.sync_user_connected(), we let the connection row
                            // do it.

                            // This method emits "::notify::user-connected", so the connection
                            // row will handle the corresponding UI events and call
                            // client.sync_user_connected().
                            client.set_connection_model(&conn);
                        }
                    ));
                },
            );
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

            let obj = self.obj();

            let controller = self.controller();
            let model = controller.clients();

            let stack = &self.stack;
            let sidebar_header_bar = &self.sidebar_header_bar;
            let nav_split_view = &self.nav_split_view;
            let sidebar = &self.sidebar;

            model.connect_notify_local(
                Some("n-items"),
                glib::clone!(
                    #[weak]
                    stack,
                    #[weak]
                    sidebar_header_bar,
                    #[weak]
                    nav_split_view,
                    move |list, _| {
                        let n_items = list.n_items();

                        stack.set_visible_child_name(if n_items != 0 {
                            "connections"
                        } else {
                            "no-connections"
                        });

                        sidebar_header_bar.set_show_title(n_items != 0);

                        if n_items == 0 {
                            nav_split_view.set_show_content(false);
                        }
                    }
                ),
            );

            gtk::ClosureExpression::new::<bool>(
                [
                    obj.property_expression_weak("display_mode"),
                    model.property_expression_weak("n-items"),
                ],
                glib::closure!(|_: Option<glib::Object>,
                                display_mode: MQTTyDisplayMode,
                                n_conns: u32| {
                    // Always collapse on mobile or when there are no connections
                    display_mode == MQTTyDisplayMode::Mobile || n_conns == 0
                }),
            )
            .bind(&**nav_split_view, "collapsed", glib::Object::NONE);

            obj.connect_display_mode_notify(glib::clone!(
                #[weak(rename_to = this)]
                self,
                #[weak]
                sidebar,
                move |_| {
                    let row = this.last_selected_row.borrow().clone();
                    sidebar.select_row(row.as_ref());
                }
            ));

            obj.property_expression_weak("display_mode")
                .chain_closure::<gtk::SelectionMode>(glib::closure!(
                    |_: Option<glib::Object>, display_mode: MQTTyDisplayMode| {
                        match display_mode {
                            // When desktop, do not allow unselections
                            MQTTyDisplayMode::Desktop => gtk::SelectionMode::Browse,
                            // When mobile, allow unselections, this could happen
                            // when pressing back the nav_split_view
                            MQTTyDisplayMode::Mobile => gtk::SelectionMode::Single,
                        }
                    }
                ))
                .bind(&**sidebar, "selection-mode", glib::Object::NONE);

            nav_split_view.connect_show_content_notify(glib::clone!(
                #[weak]
                sidebar,
                move |nav_split_view| {
                    if !nav_split_view.shows_content() {
                        sidebar.unselect_all();
                    }
                }
            ));

            sidebar.bind_model(Some(&model), |o| {
                let client = o
                    .downcast_ref::<MQTTySubscriptionMessagesClientWrapper>()
                    .unwrap();

                let row = MQTTySubscriptionsConnectionRow::new(client);

                client.connect_message(glib::clone!(
                    #[weak]
                    row,
                    move |client, msg| {
                        row.set_n_unread(row.n_unread() + 1);
                        let app = MQTTyApplication::get_singleton();
                        let window = app.active_window().and_downcast::<MQTTyWindow>().unwrap();
                        window.subscriptions_needs_attention(client.connection_model(), &msg);
                    }
                ));

                row.upcast()
            });

            sidebar.connect_row_activated(glib::clone!(
                #[weak(rename_to = this)]
                self,
                #[weak]
                obj,
                move |_, _| {
                    // We assume that ::row-selected has already being handled before this,
                    // so no need to call `self.set_subs_overview(...)`

                    if obj.display_mode() != MQTTyDisplayMode::Mobile {
                        // Handling it in ::row-selected
                        return;
                    }

                    this.nav_split_view.set_show_content(true);
                }
            ));

            sidebar.connect_row_selected(glib::clone!(
                #[weak(rename_to = this)]
                self,
                #[weak]
                obj,
                move |_, row| {
                    this.set_subs_overview(row);

                    if obj.display_mode() == MQTTyDisplayMode::Mobile {
                        // Handling it in ::row-activated
                        return;
                    }

                    this.nav_split_view.set_show_content(true);
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

            sidebar_header_bar.add_controller(click);
            sidebar_header_bar.add_controller(drag);
        }
    }
    impl WidgetImpl for MQTTySubscriptionsView {}
    impl BinImpl for MQTTySubscriptionsView {}

    impl MQTTySubscriptionsView {
        /// Remember to call `nav_split_view.set_show_content(...)` after this.
        fn set_subs_overview(&self, row: Option<&gtk::ListBoxRow>) {
            let Some(conn_row) =
                row.map(|r| r.downcast_ref::<MQTTySubscriptionsConnectionRow>().unwrap())
            else {
                // An unselection ocurred, we ignore it.
                return;
            };

            let client = conn_row.client();

            self.last_selected_row.replace(row.map(|r| r.clone()));

            let nav_split_view = &self.nav_split_view;

            let mut map = self.clients_overview_map.borrow_mut();

            let overview = map.entry(client).or_insert_with_key(|client| {
                let o = MQTTySubscriptionsOverview::new(client);
                conn_row
                    .bind_property("n_unread", &o, "n_unread")
                    .sync_create()
                    .bidirectional()
                    .build();
                o
            });

            nav_split_view.set_content(Some(overview));
        }

        pub fn controller(&self) -> &MQTTySubscriptionMessagesController {
            self.controller
                .get_or_init(|| MQTTySubscriptionMessagesController::new().unwrap())
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

        let controller = self.imp().controller();

        if controller.contains_connection(&conn.as_ref().url, &conn.client_id) {
            // The connection already exists
            toasts::client_already_exists();
            return;
        }

        controller.add_clients(&[conn.clone()]);
        let clients = controller.clients();
        let new_client = clients
            .item(clients.n_items() - 1)
            .unwrap()
            .downcast::<MQTTySubscriptionMessagesClientWrapper>()
            .unwrap();

        if conn.user_connected {
            // Let the connection row do the error handling
            new_client.notify_user_connected();
        }
    }

    // pub fn set_entries(&self, entries: &[MQTTyClient]) {
    //     let model = self.imp().model();
    //     model.splice(0, model.n_items(), entries);
    // }
}
