use std::cell::{Cell, RefCell};

use adw::subclass::prelude::*;
use gtk::glib;
use gtk::prelude::*;

use crate::gsettings::MQTTySettingConnection;
use crate::pages::MQTTyBasePage;
use crate::subclass::prelude::*;

mod imp {

    use crate::application::MQTTyApplication;

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/pages/edit_conn_page.ui")]
    #[properties(wrapper_type = super::MQTTyEditConnPage)]
    pub struct MQTTyEditConnPage {
        #[property(get, set)]
        editing: Cell<bool>,

        /// N-connection in GSettings "connections" key, used as model for this page,
        /// if nth_conn == -1, then it should be treated as a new connection to be inserted
        #[property(get, set, construct)]
        nth_conn: Cell<i64>,

        #[property(get, set)]
        conn_model: RefCell<MQTTySettingConnection>,

        #[template_child]
        url_row: TemplateChild<adw::EntryRow>,

        #[template_child]
        topic_row: TemplateChild<adw::EntryRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyEditConnPage {
        const NAME: &'static str = "MQTTyEditConnPage";

        type Type = super::MQTTyEditConnPage;

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
    impl ObjectImpl for MQTTyEditConnPage {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let app = MQTTyApplication::get_singleton();

            let conn = if obj.nth_conn() == -1 {
                Default::default()
            } else {
                obj.set_editing(true);
                app.settings_connections()[obj.nth_conn() as usize].clone()
            };

            conn.bind_property("url", &*self.url_row, "text")
                .bidirectional()
                .sync_create()
                .build();
            conn.bind_property("topic", &*self.topic_row, "text")
                .bidirectional()
                .sync_create()
                .build();

            obj.set_conn_model(conn);
        }
    }
    impl WidgetImpl for MQTTyEditConnPage {}
    impl MQTTyBasePageImpl for MQTTyEditConnPage {}
    impl NavigationPageImpl for MQTTyEditConnPage {}

    #[gtk::template_callbacks]
    impl MQTTyEditConnPage {
        #[template_callback]
        fn on_save_conn(&self) {
            let app = MQTTyApplication::get_singleton();

            let obj = self.obj();

            app.settings_set_n_connection(obj.nth_conn(), obj.conn_model());

            obj.activate_action("navigation.pop", None).unwrap();
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
    pub struct MQTTyEditConnPage(ObjectSubclass<imp::MQTTyEditConnPage>)
        @extends gtk::Widget, MQTTyBasePage, adw::NavigationPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTyEditConnPage {
    /// nth_conn is the N-connection in the GSettings key "connections" array, if -1,
    /// then a new connection will be created
    pub fn new(nav_view: &impl IsA<adw::NavigationView>, nth_conn: i64) -> Self {
        glib::Object::builder()
            .property("nav_view", nav_view)
            .property("nth_conn", nth_conn)
            .build()
    }
}
