use std::cell::{Cell, RefCell};

use adw::subclass::prelude::*;
use gtk::glib;
use gtk::prelude::*;

use crate::pages::MQTTyBasePage;
use crate::subclass::prelude::*;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/pages/edit_conn_page.ui")]
    #[properties(wrapper_type = super::MQTTyEditConnPage)]
    pub struct MQTTyEditConnPage {
        #[property(get, set)]
        host: RefCell<String>,

        #[property(get, set)]
        topic: RefCell<String>,

        #[property(get, set)]
        editing: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyEditConnPage {
        const NAME: &'static str = "MQTTyEditConnPage";

        type Type = super::MQTTyEditConnPage;

        type ParentType = MQTTyBasePage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyEditConnPage {}
    impl WidgetImpl for MQTTyEditConnPage {}
    impl MQTTyBasePageImpl for MQTTyEditConnPage {}
    impl NavigationPageImpl for MQTTyEditConnPage {}
}

glib::wrapper! {
    pub struct MQTTyEditConnPage(ObjectSubclass<imp::MQTTyEditConnPage>)
        @extends gtk::Widget, MQTTyBasePage, adw::NavigationPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
