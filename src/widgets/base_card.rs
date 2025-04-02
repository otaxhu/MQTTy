use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/base_card.ui")]
    #[properties(wrapper_type = super::MQTTyBaseCard)]
    pub struct MQTTyBaseCard {
        #[property(get, set)]
        prefix_widget: RefCell<Option<gtk::Widget>>,

        #[property(get, set)]
        title: RefCell<String>,

        #[property(get, set)]
        subtitle: RefCell<String>,

        #[template_child]
        title_label: TemplateChild<gtk::Label>,

        #[template_child]
        subtitle_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyBaseCard {
        const NAME: &'static str = "MQTTyBaseCard";

        type Type = super::MQTTyBaseCard;

        type ParentType = gtk::FlowBoxChild;

        const ABSTRACT: bool = true;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyBaseCard {
        fn constructed(&self) {
            self.parent_constructed();

            let show_label = gtk::PropertyExpression::new(
                gtk::Label::static_type(),
                gtk::Expression::NONE, // Queries the property on binded "this" object
                "label",
            )
            .chain_closure::<bool>(glib::closure!(|_: gtk::Label, label: String| {
                label != ""
            }));

            show_label.bind(
                &*self.subtitle_label,
                "visible",
                Some(&*self.subtitle_label),
            );
            show_label.bind(&*self.title_label, "visible", Some(&*self.title_label));
        }
    }
    impl WidgetImpl for MQTTyBaseCard {}
    impl FlowBoxChildImpl for MQTTyBaseCard {}
}

glib::wrapper! {
    pub struct MQTTyBaseCard(ObjectSubclass<imp::MQTTyBaseCard>)
        @extends gtk::Widget, gtk::FlowBoxChild,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

pub trait MQTTyBaseCardImpl: FlowBoxChildImpl {}

unsafe impl<T: MQTTyBaseCardImpl> IsSubclassable<T> for MQTTyBaseCard {}
