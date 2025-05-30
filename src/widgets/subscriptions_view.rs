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
mod subscriptions_connections;

pub use connection_dialog::MQTTySubscriptionsConnectionDialog;
pub use connection_row::MQTTySubscriptionsConnectionRow;
pub use subscriptions_connections::MQTTySubscriptionsConnections;

use std::cell::OnceCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::{application::MQTTyApplication, client::MQTTyClientSubscriptionsData};

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/subscriptions_view.ui")]
    #[properties(wrapper_type = super::MQTTySubscriptionsView)]
    pub struct MQTTySubscriptionsView {
        #[property(get = Self::model)]
        model: OnceCell<gio::ListStore>,

        #[template_child]
        nav_split_view: TemplateChild<adw::NavigationSplitView>,

        #[template_child]
        connections: TemplateChild<MQTTySubscriptionsConnections>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionsView {
        const NAME: &'static str = "MQTTySubscriptionsView";

        type Type = super::MQTTySubscriptionsView;

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
    impl ObjectImpl for MQTTySubscriptionsView {}
    impl WidgetImpl for MQTTySubscriptionsView {}
    impl BinImpl for MQTTySubscriptionsView {}

    impl MQTTySubscriptionsView {
        fn model(&self) -> gio::ListStore {
            self.model
                .get_or_init(|| gio::ListStore::new::<MQTTyClientSubscriptionsData>())
                .clone()
        }
    }

    #[gtk::template_callbacks]
    impl MQTTySubscriptionsView {
        #[template_callback]
        fn connection_activated(&self, row: &MQTTySubscriptionsConnectionRow) {}
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

        let model = self.model();

        model.append(&data);
    }

    // pub fn set_entries(&self, entries: &[MQTTyClient]) {
    //     let model = self.imp().model();
    //     model.splice(0, model.n_items(), entries);
    // }
}
