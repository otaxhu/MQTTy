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
mod subscription_dialog;
mod subscription_messages;
mod subscription_row;
mod subscriptions_overview;

pub use connection_dialog::MQTTySubscriptionsConnectionDialog;
pub use connection_row::MQTTySubscriptionsConnectionRow;
pub use subscription_dialog::MQTTySubscriptionDialog;
pub use subscription_messages::MQTTySubscriptionMessages;
pub use subscription_row::MQTTySubscriptionRow;
pub use subscriptions_overview::MQTTySubscriptionsOverview;

use std::cell::{Cell, OnceCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::application::MQTTyApplication;
use crate::client::MQTTyClientSubscriptionsData;
use crate::display_mode::{MQTTyDisplayMode, MQTTyDisplayModeIface, MQTTyDisplayModeIfaceImpl};

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/subscriptions_view.ui")]
    #[properties(wrapper_type = super::MQTTySubscriptionsView)]
    pub struct MQTTySubscriptionsView {
        model: OnceCell<gio::ListStore>,

        #[property(get, set, builder(Default::default()))]
        display_mode: Cell<MQTTyDisplayMode>,

        #[template_child]
        nav_split_view: TemplateChild<adw::NavigationSplitView>,

        #[template_child]
        list_box: TemplateChild<gtk::ListBox>,

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
            klass.bind_template_callbacks();
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

            self.list_box.bind_model(
                Some(model),
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

            let obj = self.obj();

            let list_box = &self.list_box;
            let nav_split_view = &self.nav_split_view;

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

            nav_split_view.connect_show_content_notify(glib::clone!(
                #[weak]
                list_box,
                move |nav_split_view| {
                    // So this should handle clicking the Back button, and together with the
                    // code inside of on_connection_activated, should cover also the
                    // unselecting thing
                    //
                    // Should also cover when you edit or delete a selected connection
                    if !nav_split_view.shows_content() {
                        if let Some(selected) = list_box.selected_row() {
                            list_box.unselect_all();
                            selected.set_selectable(false);
                        }
                    }
                }
            ));
        }
    }
    impl WidgetImpl for MQTTySubscriptionsView {}
    impl BinImpl for MQTTySubscriptionsView {}

    impl MQTTySubscriptionsView {
        pub fn model(&self) -> &gio::ListStore {
            self.model
                .get_or_init(|| gio::ListStore::new::<MQTTyClientSubscriptionsData>())
        }

        pub fn new_connection_row_with_signals(
            &self,
            data: &MQTTyClientSubscriptionsData,
        ) -> MQTTySubscriptionsConnectionRow {
            let row = MQTTySubscriptionsConnectionRow::from(data);
            row.connect_delete_request(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |row| {
                    let model = this.model();

                    if row.is_selected() {
                        this.nav_split_view.set_show_content(false);
                    }

                    model.remove(row.index() as u32);
                }
            ));
            row.connect_edit_request(glib::clone!(
                #[weak(rename_to = this)]
                self,
                #[weak]
                data,
                move |row| {
                    glib::spawn_future_local(glib::clone!(
                        #[weak]
                        row,
                        async move {
                            let dialog =
                                MQTTySubscriptionsConnectionDialog::new_edit(&data.connection());

                            let app = MQTTyApplication::get_singleton();
                            let window = app.active_window().unwrap();
                            let Some(conn) = dialog.choose_future(&window).await else {
                                return;
                            };

                            let model = this.model();

                            let new_data = MQTTyClientSubscriptionsData::new();
                            new_data.set_connection(&conn);
                            new_data.set_subscriptions(&data.subscriptions());

                            if row.is_selected() {
                                this.nav_split_view.set_show_content(false);
                            }

                            model.splice(row.index() as u32, 1, &[new_data]);
                        }
                    ));
                }
            ));
            row
        }
    }

    impl MQTTyDisplayModeIfaceImpl for MQTTySubscriptionsView {}

    #[gtk::template_callbacks]
    impl MQTTySubscriptionsView {
        #[template_callback]
        fn on_connection_activated(&self, row: &MQTTySubscriptionsConnectionRow) {
            let list_box = &self.list_box;
            let nav_split_view = &self.nav_split_view;
            if row.is_selected() {
                // Calls the above notify handler
                nav_split_view.set_show_content(false);
                return;
            }
            list_box.selected_row().map(|row| row.set_selectable(false));
            list_box.unselect_all();
            row.set_selectable(true);
            list_box.select_row(Some(row));
            nav_split_view.set_content(Some(&MQTTySubscriptionsOverview::from(row.data())));
            nav_split_view.set_show_content(true);
        }
    }
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
