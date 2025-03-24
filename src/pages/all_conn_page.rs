use adw::subclass::prelude::*;
use gtk::glib;
use gtk::prelude::*;

use crate::pages::MQTTyBasePage;
use crate::pages::MQTTyEditConnPage;
use crate::subclass::prelude::*;
use crate::widgets::MQTTyAddConnCard;
use crate::widgets::MQTTyBaseCard;
use crate::widgets::MQTTyConnCard;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/pages/all_conn_page.ui")]
    pub struct MQTTyAllConnPage {
        #[template_child]
        flowbox: TemplateChild<gtk::FlowBox>,

        #[template_child]
        add_conn_card: TemplateChild<MQTTyAddConnCard>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyAllConnPage {
        const NAME: &'static str = "MQTTyAllConnPage";

        type Type = super::MQTTyAllConnPage;

        type ParentType = MQTTyBasePage;

        fn class_init(klass: &mut Self::Class) {
            MQTTyAddConnCard::static_type();
            MQTTyConnCard::static_type();

            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for MQTTyAllConnPage {
        fn constructed(&self) {
            self.parent_constructed();

            let add_conn_card = &self.add_conn_card;

            let obj = self.obj();

            let nav_view = obj.upcast_ref::<MQTTyBasePage>().nav_view();

            self.flowbox.connect_child_activated(glib::clone!(
                #[weak]
                add_conn_card,
                move |_, child| {
                    if child == add_conn_card.upcast_ref::<gtk::FlowBoxChild>() {
                        // TODO: bind/pass a model
                        nav_view.push(&MQTTyEditConnPage::new(&nav_view));
                    }
                }
            ));
        }
    }
    impl WidgetImpl for MQTTyAllConnPage {}
    impl NavigationPageImpl for MQTTyAllConnPage {}
    impl MQTTyBasePageImpl for MQTTyAllConnPage {}

    impl MQTTyAllConnPage {
        fn append_card(&self, card: &impl IsA<MQTTyBaseCard>) {
            self.flowbox.append(card.upcast_ref());
        }

        fn insert_card(&self, card: &impl IsA<MQTTyBaseCard>, position: i32) {
            self.flowbox.insert(card.upcast_ref(), position);
        }
    }
}

glib::wrapper! {
    pub struct MQTTyAllConnPage(ObjectSubclass<imp::MQTTyAllConnPage>)
    @extends gtk::Widget, adw::NavigationPage, MQTTyBasePage,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
