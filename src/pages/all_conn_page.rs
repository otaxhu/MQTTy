// Copyright (c) 2025 Oscar Pernia
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::application::MQTTyApplication;
use crate::gsettings::MQTTySettingConnection;
use crate::pages::MQTTyAddConnPage;
use crate::pages::MQTTyBasePage;
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

            store.append(&add_conn_card);

            // DO NOT CHANGE ORDER OF CALL
            //
            // See: https://docs.gtk.org/gio/signal.Settings.changed.html#description
            app.settings_connections()
                .connect_items_changed(glib::clone!(
                    #[weak]
                    store,
                    move |_, _, _, _| {
                        let app = MQTTyApplication::get_singleton();
                        let conns = app.settings_connections();

                        store.splice(
                            1,
                            store.n_items() - 1,
                            &conns
                                .into_iter()
                                .map(|i| i.unwrap().downcast::<MQTTySettingConnection>().unwrap())
                                .map(MQTTyConnCard::from)
                                .collect::<Vec<_>>(),
                        );
                    }
                ));

            // DO NOT CHANGE ORDER OF CALL
            //
            // See: https://docs.gtk.org/gio/signal.Settings.changed.html#description
            store.extend(
                app.settings_connections()
                    .into_iter()
                    .map(|i| i.unwrap().downcast::<MQTTySettingConnection>().unwrap())
                    .map(MQTTyConnCard::from),
            );

            flowbox.bind_model(Some(&store), |obj| {
                obj.downcast_ref::<gtk::Widget>().unwrap().clone()
            });

            flowbox.connect_child_activated(move |_, c| {
                let i = c.index();

                if i == 0 {
                    nav_view.push(&MQTTyAddConnPage::new(&nav_view));
                    return;
                }

                nav_view.push(&MQTTyPanelPage::new(&nav_view, (i - 1) as u32));
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
