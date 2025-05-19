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

mod subscriptions_row;

pub use subscriptions_row::MQTTySubscriptionsRow;

use std::cell::OnceCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::client::MQTTyClient;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/subscriptions_view.ui")]
    pub struct MQTTySubscriptionsView {
        model: OnceCell<gio::ListStore>,

        #[template_child]
        stack: TemplateChild<gtk::Stack>,

        #[template_child]
        pub list_box: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionsView {
        const NAME: &'static str = "MQTTySubscriptionsView";

        type Type = super::MQTTySubscriptionsView;

        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MQTTySubscriptionsView {
        fn constructed(&self) {
            self.parent_constructed();

            let model = self.model();

            let stack = &self.stack;

            model.connect_notify_local(
                Some("n-items"),
                glib::clone!(
                    #[weak]
                    stack,
                    move |model, _| {
                        stack.set_visible_child_name(if model.n_items() != 0 {
                            "subscriptions"
                        } else {
                            "no-subscriptions"
                        });
                    }
                ),
            );

            self.list_box.connect_row_activated(|list_box, row| {
                // TODO: Show the bottom sheet with extra options like deleting
                // the subscription, and of course, all of the messages listed
                // for this subscription
            });

            self.list_box.bind_model(Some(model), |o| {
                let client = o.downcast_ref::<MQTTyClient>().unwrap();
                let row = MQTTySubscriptionsRow::new();
                row.add_css_class("subscriptions-row");
                row.set_prefix_widget(
                    gtk::Box::builder()
                        .css_classes(["success", "indicator", "circular"])
                        .valign(gtk::Align::Center)
                        .build(),
                );
                // TODO
                // row.set_title(...);
                // row.set_subtitle(...);
                row.upcast()
            });
        }
    }
    impl WidgetImpl for MQTTySubscriptionsView {}
    impl BinImpl for MQTTySubscriptionsView {}

    impl MQTTySubscriptionsView {
        pub fn model(&self) -> &gio::ListStore {
            self.model
                .get_or_init(|| gio::ListStore::new::<MQTTyClient>())
        }
    }
}

glib::wrapper! {
    pub struct MQTTySubscriptionsView(ObjectSubclass<imp::MQTTySubscriptionsView>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTySubscriptionsView {
    pub fn new_subscription(&self) {
        let imp = self.imp();
        let model = imp.model();
        model.append(&MQTTyClient::default());
        let list_box = &*imp.list_box;
        list_box
            .row_at_index((model.n_items() - 1) as i32)
            .unwrap()
            .downcast::<MQTTySubscriptionsRow>()
            .unwrap()
            .set_editing(true);
    }

    pub fn set_entries(&self, entries: &[MQTTyClient]) {
        let model = self.imp().model();
        model.splice(0, model.n_items(), entries);
    }
}
