use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/publish_parameters.ui")]
    #[properties(wrapper_type = super::MQTTyPublishParameters)]
    pub struct MQTTyPublishParameters {
        #[property(get, set)]
        mqtt_version: RefCell<String>,

        #[template_child]
        mqtt_3_button: TemplateChild<gtk::CheckButton>,

        #[template_child]
        mqtt_5_button: TemplateChild<gtk::CheckButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyPublishParameters {
        const NAME: &'static str = "MQTTyPublishParameters";

        type Type = super::MQTTyPublishParameters;

        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.install_property_action("publish-parameters.mqtt-version", "mqtt_version");
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyPublishParameters {
        fn constructed(&self) {
            self.parent_constructed();

            self.mqtt_3_button.set_action_target(Some("3"));
            self.mqtt_5_button.set_action_target(Some("5"));
        }
    }
    impl WidgetImpl for MQTTyPublishParameters {}
    impl BinImpl for MQTTyPublishParameters {}
}

glib::wrapper! {
    pub struct MQTTyPublishParameters(ObjectSubclass<imp::MQTTyPublishParameters>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
