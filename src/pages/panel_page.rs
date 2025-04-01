use std::cell::Cell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

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
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyPanelPage {
        const NAME: &'static str = "MQTTyPanelPage";

        type Type = super::MQTTyPanelPage;

        type ParentType = MQTTyBasePage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyPanelPage {}
    impl WidgetImpl for MQTTyPanelPage {}
    impl NavigationPageImpl for MQTTyPanelPage {}
    impl MQTTyBasePageImpl for MQTTyPanelPage {}
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
