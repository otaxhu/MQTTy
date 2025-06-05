use adw::subclass::prelude::*;
use gtk::glib;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/subscription_messages.ui")]
    pub struct MQTTySubscriptionMessages {}

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionMessages {
        const NAME: &'static str = "MQTTySubscriptionMessages";

        type Type = super::MQTTySubscriptionMessages;

        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MQTTySubscriptionMessages {}
    impl WidgetImpl for MQTTySubscriptionMessages {}
    impl BinImpl for MQTTySubscriptionMessages {}
}

glib::wrapper! {
    pub struct MQTTySubscriptionMessages(ObjectSubclass<imp::MQTTySubscriptionMessages>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTySubscriptionMessages {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}
