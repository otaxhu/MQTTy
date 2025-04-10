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

use std::cell::{Cell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use crate::application::MQTTyApplication;
use crate::gsettings::MQTTySettingConnection;
use crate::pages::MQTTyBasePage;
use crate::subclass::prelude::*;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/pages/panel_page.ui")]
    #[properties(wrapper_type = super::MQTTyPanelPage)]
    pub struct MQTTyPanelPage {
        /// N-connection in GSettings "connections" key, used for retrieving the connection
        /// data, it's unsigned integer because we expect to retrieve always valid data
        #[property(get, set, construct)]
        nth_conn: Cell<u32>,

        #[property(get, set)]
        conn_model: RefCell<MQTTySettingConnection>,

        #[template_child]
        view_stack: TemplateChild<adw::ViewStack>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyPanelPage {
        const NAME: &'static str = "MQTTyPanelPage";

        type Type = super::MQTTyPanelPage;

        type ParentType = MQTTyBasePage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyPanelPage {
        fn constructed(&self) {
            self.parent_constructed();

            let app = MQTTyApplication::get_singleton();

            let obj = self.obj();

            let conn_model = app.settings_n_connection(obj.nth_conn()).unwrap();

            obj.upcast_ref::<adw::NavigationPage>()
                .set_title(&conn_model.topic());

            self.view_stack
                .connect_visible_child_name_notify(glib::clone!(
                    #[weak]
                    obj,
                    move |view_stack| {
                        obj.upcast_ref::<MQTTyBasePage>()
                            .top_end_widget()
                            .unwrap()
                            .set_visible(view_stack.visible_child_name().unwrap() == "publish");
                    }
                ));

            obj.set_conn_model(conn_model);
        }
    }
    impl WidgetImpl for MQTTyPanelPage {}
    impl NavigationPageImpl for MQTTyPanelPage {}
    impl MQTTyBasePageImpl for MQTTyPanelPage {}

    #[gtk::template_callbacks]
    impl MQTTyPanelPage {
        #[template_callback]
        fn on_save_conn(&self) {
            let app = MQTTyApplication::get_singleton();

            let obj = self.obj();

            let conn_model = obj.conn_model();

            app.settings_set_n_connection(obj.nth_conn().into(), conn_model.clone());

            obj.upcast_ref::<adw::NavigationPage>()
                .set_title(&conn_model.topic());
        }

        #[template_callback]
        fn on_delete_conn(&self) {
            let app = MQTTyApplication::get_singleton();

            let obj = self.obj();

            app.settings_delete_n_connection(obj.nth_conn());

            obj.activate_action("navigation.pop", None).unwrap();
        }
    }
}

glib::wrapper! {
    pub struct MQTTyPanelPage(ObjectSubclass<imp::MQTTyPanelPage>)
        @extends gtk::Widget, adw::NavigationPage, MQTTyBasePage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTyPanelPage {
    pub fn new(nav_view: &impl IsA<adw::NavigationView>, nth_conn: u32) -> Self {
        glib::Object::builder()
            .property("nav_view", nav_view)
            .property("nth_conn", nth_conn)
            .build()
    }
}
