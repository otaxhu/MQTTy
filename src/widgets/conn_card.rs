use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use crate::gsettings::MQTTyOpenConnection;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/conn_card.ui")]
    #[properties(wrapper_type = super::MQTTyConnCard)]
    pub struct MQTTyConnCard {
        #[property(get, set)]
        host: RefCell<String>,

        #[property(get, set)]
        topic: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyConnCard {
        const NAME: &'static str = "MQTTyConnCard";

        type Type = super::MQTTyConnCard;

        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MQTTyConnCard {}
    impl WidgetImpl for MQTTyConnCard {}
    impl BinImpl for MQTTyConnCard {}
}

glib::wrapper! {
    pub struct MQTTyConnCard(ObjectSubclass<imp::MQTTyConnCard>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTyConnCard {
    pub fn new(host: &String, topic: &String) -> Self {
        glib::Object::builder()
            .property("host", host)
            .property("topic", topic)
            .build()
    }
}

// Helper method for instantiating a MQTTyConnCard from a GSetting MQTTyOpenConnection
impl From<&MQTTyOpenConnection> for MQTTyConnCard {
    fn from(value: &MQTTyOpenConnection) -> Self {
        Self::new(&value.host, &value.topic)
    }
}
