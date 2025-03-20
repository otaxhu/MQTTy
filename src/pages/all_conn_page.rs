use adw::subclass::prelude::*;
use gtk::glib;

use crate::pages::prelude::*;
use crate::pages::MQTTyBasePage;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/pages/all_conn_page.ui")]
    pub struct MQTTyAllConnPage {}

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyAllConnPage {
        const NAME: &'static str = "MQTTyAllConnPage";

        type Type = super::MQTTyAllConnPage;

        type ParentType = MQTTyBasePage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for MQTTyAllConnPage {}
    impl WidgetImpl for MQTTyAllConnPage {}
    impl NavigationPageImpl for MQTTyAllConnPage {}
    impl MQTTyBasePageImpl for MQTTyAllConnPage {}
}

glib::wrapper! {
    pub struct MQTTyAllConnPage(ObjectSubclass<imp::MQTTyAllConnPage>)
        @extends gtk::Widget, adw::NavigationPage, MQTTyBasePage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
