use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::application::MQTTyApplication;
use crate::gsettings::MQTTySettingConnection;
use crate::pages::MQTTyBasePage;
use crate::pages::MQTTyEditConnPage;
use crate::pages::MQTTyPanelPage;
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
    }

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
    impl ObjectImpl for MQTTyAllConnPage {
        fn constructed(&self) {
            self.parent_constructed();

            let add_conn_card = MQTTyAddConnCard::new();

            let app = MQTTyApplication::get_singleton();

            let flowbox = &self.flowbox;

            let mut store = gio::ListStore::new::<MQTTyBaseCard>();

            let obj = self.obj();

            let nav_view = obj.upcast_ref::<MQTTyBasePage>().nav_view();

            let conn_card_from_settings_callback = glib::clone!(
                #[weak_allow_none]
                nav_view,
                move |(i, c): (usize, MQTTySettingConnection)| {
                    let nav_view = nav_view.unwrap();
                    let card = MQTTyConnCard::from(c);
                    card.connect_local("edit-button-clicked", false, move |_| {
                        nav_view.push(&MQTTyEditConnPage::new(&nav_view, i as i64));
                        None
                    });
                    card
                }
            );

            store.append(&add_conn_card);

            // DO NOT CHANGE ORDER OF CALL
            //
            // See: https://docs.gtk.org/gio/signal.Settings.changed.html#description
            app.settings().connect_changed(
                Some("connections"),
                glib::clone!(
                    #[weak]
                    store,
                    #[strong]
                    conn_card_from_settings_callback,
                    move |_, _| {
                        let app = MQTTyApplication::get_singleton();
                        let conns = app.settings_connections();

                        store.splice(
                            1,
                            store.n_items() - 1,
                            &conns
                                .into_iter()
                                .enumerate()
                                .map(conn_card_from_settings_callback.clone())
                                .collect::<Vec<_>>(),
                        );
                    }
                ),
            );

            // DO NOT CHANGE ORDER OF CALL
            //
            // See: https://docs.gtk.org/gio/signal.Settings.changed.html#description
            store.extend(
                app.settings_connections()
                    .into_iter()
                    .enumerate()
                    .map(conn_card_from_settings_callback),
            );

            flowbox.bind_model(Some(&store), |obj| {
                obj.downcast_ref::<gtk::Widget>().unwrap().clone()
            });

            flowbox.connect_child_activated(move |_, c| {
                let i = c.index();

                if i == 0 {
                    nav_view.push(&MQTTyEditConnPage::new(&nav_view, -1));
                    return;
                }

                nav_view.push(&MQTTyPanelPage::new(&nav_view, i as u32));
            });
        }
    }
    impl WidgetImpl for MQTTyAllConnPage {}
    impl NavigationPageImpl for MQTTyAllConnPage {}
    impl MQTTyBasePageImpl for MQTTyAllConnPage {}
}

glib::wrapper! {
    pub struct MQTTyAllConnPage(ObjectSubclass<imp::MQTTyAllConnPage>)
    @extends gtk::Widget, adw::NavigationPage, MQTTyBasePage,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
