use std::sync::LazyLock;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::glib::subclass::Signal;

use crate::gsettings::MQTTySettingConnection;
use crate::subclass::prelude::*;
use crate::widgets::MQTTyBaseCard;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/conn_card.ui")]
    pub struct MQTTyConnCard {}

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyConnCard {
        const NAME: &'static str = "MQTTyConnCard";

        type Type = super::MQTTyConnCard;

        type ParentType = MQTTyBaseCard;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MQTTyConnCard {
        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> =
                LazyLock::new(|| vec![Signal::builder("edit-button-clicked").build()]);
            &*SIGNALS
        }
    }
    impl WidgetImpl for MQTTyConnCard {}
    impl FlowBoxChildImpl for MQTTyConnCard {}
    impl MQTTyBaseCardImpl for MQTTyConnCard {}

    #[gtk::template_callbacks]
    impl MQTTyConnCard {
        #[template_callback]
        fn edit_button_clicked(&self) {
            self.obj().emit_by_name::<()>("edit-button-clicked", &[]);
        }
    }
}

glib::wrapper! {
    /// Emits "edit-button-clicked" signal when the edit connection button is clicked
    pub struct MQTTyConnCard(ObjectSubclass<imp::MQTTyConnCard>)
        @extends gtk::Widget, gtk::FlowBoxChild, MQTTyBaseCard,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTyConnCard {
    pub fn new(host: &String, topic: &String) -> Self {
        glib::Object::builder::<Self>()
            .property("subtitle", host)
            .property("title", topic)
            .build()
    }
}

impl Default for MQTTyConnCard {
    fn default() -> Self {
        Self::new(&"".to_string(), &"".to_string())
    }
}

// Helper method for instantiating a MQTTyConnCard from a GSetting MQTTyOpenConnection
impl From<MQTTySettingConnection> for MQTTyConnCard {
    fn from(value: MQTTySettingConnection) -> Self {
        // TODO: Extract host and pass as parameter instead of whole url
        Self::new(&value.url(), &value.topic())
    }
}
