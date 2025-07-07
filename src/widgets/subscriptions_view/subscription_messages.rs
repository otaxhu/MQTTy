use std::cell::OnceCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::client::MQTTyClientMessage;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/subscription_messages.ui")]
    pub struct MQTTySubscriptionMessages {
        model: OnceCell<gio::ListStore>,

        #[template_child]
        stack: TemplateChild<gtk::Stack>,

        #[template_child]
        list_box: TemplateChild<gtk::ListBox>,
    }

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

    impl ObjectImpl for MQTTySubscriptionMessages {
        fn constructed(&self) {
            self.parent_constructed();

            let stack = &self.stack;

            let model = self.model();
            model.connect_notify_local(
                Some("n-items"),
                glib::clone!(
                    #[weak]
                    stack,
                    move |model, _| {
                        stack.set_visible_child_name(if model.n_items() != 0 {
                            "messages"
                        } else {
                            "no-messages"
                        });
                    }
                ),
            );

            let list_box = &self.list_box;

            list_box.bind_model(Some(model), |o| todo!());
        }
    }
    impl WidgetImpl for MQTTySubscriptionMessages {}
    impl BinImpl for MQTTySubscriptionMessages {}

    impl MQTTySubscriptionMessages {
        pub fn model(&self) -> &gio::ListStore {
            self.model
                .get_or_init(|| gio::ListStore::new::<MQTTyClientMessage>())
        }
    }
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

    pub fn set_messages(&self, messages: &[MQTTyClientMessage]) {
        let model = self.imp().model();
        model.splice(0, model.n_items(), messages);
    }

    pub fn append_message(&self, message: &MQTTyClientMessage) {
        let model = self.imp().model();
        model.append(message);
    }
}
