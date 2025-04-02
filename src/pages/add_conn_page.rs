use std::cell::RefCell;

use adw::subclass::prelude::*;
use gtk::glib;
use gtk::prelude::*;

use crate::application::MQTTyApplication;
use crate::gsettings::MQTTySettingConnection;
use crate::pages::MQTTyBasePage;
use crate::subclass::prelude::*;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/pages/add_conn_page.ui")]
    #[properties(wrapper_type = super::MQTTyAddConnPage)]
    pub struct MQTTyAddConnPage {
        #[property(get, set)]
        conn_model: RefCell<MQTTySettingConnection>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyAddConnPage {
        const NAME: &'static str = "MQTTyAddConnPage";

        type Type = super::MQTTyAddConnPage;

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
    impl ObjectImpl for MQTTyAddConnPage {}
    impl WidgetImpl for MQTTyAddConnPage {}
    impl MQTTyBasePageImpl for MQTTyAddConnPage {}
    impl NavigationPageImpl for MQTTyAddConnPage {}

    #[gtk::template_callbacks]
    impl MQTTyAddConnPage {
        #[template_callback]
        fn on_save_conn(&self) {
            let app = MQTTyApplication::get_singleton();

            let obj = self.obj();

            app.settings_set_n_connection(-1, obj.conn_model());

            obj.activate_action("navigation.pop", None).unwrap();
        }
    }
}

glib::wrapper! {
    pub struct MQTTyAddConnPage(ObjectSubclass<imp::MQTTyAddConnPage>)
        @extends gtk::Widget, MQTTyBasePage, adw::NavigationPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTyAddConnPage {
    pub fn new(nav_view: &impl IsA<adw::NavigationView>) -> Self {
        glib::Object::builder()
            .property("nav_view", nav_view)
            .build()
    }
}
