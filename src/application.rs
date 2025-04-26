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

use std::cell::{OnceCell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{gio, glib};

use crate::client::MQTTyClient;
use crate::config;
use crate::display_mode::{MQTTyDisplayMode, MQTTyDisplayModeIface};
use crate::gsettings::MQTTySettingConnection;
use crate::main_window::MQTTyWindow;
use crate::pages::{MQTTyAddConnPage, MQTTyAllConnPage, MQTTyBasePage, MQTTyPanelPage};
use crate::widgets::{
    MQTTyAddConnCard, MQTTyBaseCard, MQTTyConnCard, MQTTyEditConnListBox, MQTTyKeyValueRow,
    MQTTyPublishBodyTab, MQTTyPublishGeneralTab, MQTTyPublishUserPropsTab, MQTTyPublishView,
    MQTTySourceView,
};

mod imp {

    use super::*;

    #[derive(Default)]
    pub struct MQTTyApplication {
        pub settings: OnceCell<gio::Settings>,

        /// The type of items inside of ListStore is MQTTySettingConnection
        pub settings_conns: OnceCell<gio::ListStore>,

        /// This clients are index mapped 1-to-1 to the settings_conns, they are separated
        /// because they cannot be tupled and passed to a gio::ListStore, and also because
        /// we are disconnecting the corresponding client if any MQTTySettingConnection was
        /// deleted, because the settings_conns::items-changed passes the already mutated list,
        /// we have to search for the mqtt connection in this Vec, disconnect it and remove it.
        ///
        /// IMPORTANT:
        ///
        /// Listen to settings_conns::items-changed signal for connections removals or
        /// connections additions, and act accordingly (by disconnecting the MQTT client or
        /// connecting a new one, respectively)
        pub clients: Rc<RefCell<Vec<MQTTyClient>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyApplication {
        const NAME: &'static str = "MQTTyApplication";
        type Type = super::MQTTyApplication;
        type ParentType = adw::Application;

        fn class_init(_klass: &mut Self::Class) {
            // Eagerly initialize everything

            MQTTyWindow::static_type();
            MQTTySettingConnection::static_type();

            // Widgets
            MQTTyBaseCard::static_type();
            MQTTyAddConnCard::static_type();
            MQTTyConnCard::static_type();
            MQTTyEditConnListBox::static_type();
            MQTTySourceView::static_type();
            MQTTyKeyValueRow::static_type();

            MQTTyPublishView::static_type();
            MQTTyPublishGeneralTab::static_type();
            MQTTyPublishBodyTab::static_type();
            MQTTyPublishUserPropsTab::static_type();

            // Pages
            MQTTyBasePage::static_type();
            MQTTyAllConnPage::static_type();
            MQTTyAddConnPage::static_type();
            MQTTyPanelPage::static_type();

            // Enums
            MQTTyDisplayMode::static_type();

            // Interfaces
            MQTTyDisplayModeIface::static_type();
        }
    }

    impl ObjectImpl for MQTTyApplication {}

    impl ApplicationImpl for MQTTyApplication {
        fn activate(&self) {
            self.parent_activate();
            let app = self.obj();

            if let Some(window) = app.active_window() {
                window.present();
                return;
            }

            let window = MQTTyWindow::new(&app);

            window.present();
        }

        fn startup(&self) {
            self.parent_startup();

            sourceview::init();

            let app = self.obj();

            // Set icons for shell
            gtk::Window::set_default_icon_name(config::APP_ID);

            app.setup_css();
            app.setup_gactions();
            app.setup_accels();

            app.setup_settings();
        }
    }

    impl GtkApplicationImpl for MQTTyApplication {}
    impl AdwApplicationImpl for MQTTyApplication {}
}

glib::wrapper! {
    pub struct MQTTyApplication(ObjectSubclass<imp::MQTTyApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl MQTTyApplication {
    pub fn get_singleton() -> Self {
        match gio::Application::default().and_downcast::<MQTTyApplication>() {
            None => glib::Object::builder()
                .property("application-id", config::APP_ID)
                .property("resource-base-path", "/io/github/otaxhu/MQTTy/")
                .build(),
            Some(app) => app,
        }
    }

    pub fn settings(&self) -> &gio::Settings {
        self.imp()
            .settings
            .get_or_init(|| gio::Settings::new(config::APP_ID))
    }

    pub fn settings_connections(&self) -> &gio::ListStore {
        self.imp()
            .settings_conns
            .get_or_init(|| gio::ListStore::new::<MQTTySettingConnection>())
    }

    pub fn settings_n_connection(&self, n: u32) -> Option<MQTTySettingConnection> {
        self.settings_connections()
            .item(n)
            .map(|o| o.downcast::<MQTTySettingConnection>().unwrap())
    }

    pub fn settings_set_n_connection(&self, n: i64, conn: MQTTySettingConnection) {
        let conns = self.settings_connections();
        if n == -1 {
            conns.append(&conn);
        } else {
            conns.splice(n as u32, 1, &[conn]);
        }
    }

    pub fn settings_delete_n_connection(&self, n: u32) {
        let conns = self.settings_connections();
        conns.remove(n);
    }

    pub fn clients(&self) -> &Rc<RefCell<Vec<MQTTyClient>>> {
        &self.imp().clients
    }

    /// We are only requesting the GSettings on startup to prevent infinite recursion,
    /// e.g. app.settings_connections()::items-changed it's emitted, it is saved to
    /// external GSettings, GSettings::changed it's emitted, app.settings_connections() gets
    /// updated with external settings,
    /// app.settings_connections()::items-changed it's emitted again, etc.
    fn setup_settings(&self) {
        // let settings = self.settings();
        //
        // let external_conns = settings.get::<Vec<MQTTySettingConnection>>("connections");
        //
        // let app_conns = self.settings_connections();
        //
        // app_conns.extend_from_slice(&external_conns);
        //
        // let clients_ref = self.clients();
        //
        // let mut clients_mut = clients_ref.borrow_mut();
        // clients_mut.reserve(external_conns.len());
        //
        // for conn in app_conns
        //     .iter::<MQTTySettingConnection>()
        //     .map(|i| i.unwrap())
        // {
        //     let connection = MQTTyClient::new(&conn);
        //
        //     clients_mut.push(connection);
        // }
        //
        // // Save settings to external GSettings, and creating MQTT clients for each one
        // app_conns.connect_items_changed(glib::clone!(
        //     #[strong]
        //     settings,
        //     #[strong]
        //     clients_ref,
        //     move |list: &gio::ListStore, pos, rem, add| {
        //         let mut clients_mut = clients_ref.borrow_mut();
        //
        //         settings
        //             .set(
        //                 "connections",
        //                 list.iter::<MQTTySettingConnection>()
        //                     .map(|i| i.unwrap().downcast::<MQTTySettingConnection>().unwrap())
        //                     .collect::<Vec<_>>(),
        //             )
        //             .unwrap();
        //
        //         let pos = pos as usize;
        //         let rem = rem as usize;
        //         let add = add as usize;
        //
        //         // Removals
        //         for client in clients_mut.splice(pos..pos + rem, None) {
        //             client.disconnect_client();
        //         }
        //
        //         // Additions
        //         clients_mut.reserve(add);
        //
        //         for i in pos..pos + add {
        //             let new_client = MQTTyClient::new(
        //                 &list
        //                     .item(i as u32)
        //                     .unwrap()
        //                     .downcast::<MQTTySettingConnection>()
        //                     .unwrap(),
        //             );
        //
        //             clients_mut.insert(i, new_client);
        //         }
        //     }
        // ));
    }

    fn setup_gactions(&self) {
        // Quit
        let action_quit = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| {
                // This is needed to trigger the delete event and saving the window state
                if let Some(win) = app.active_window() {
                    win.set_hide_on_close(false);
                    win.close();
                }
                app.quit();
            })
            .build();

        let action_about = gio::ActionEntry::builder("about")
            .activate(|app: &Self, _, _| {
                let about_dialog = adw::AboutDialog::builder()
                    .application_name("MQTTy")
                    .application_icon(config::APP_ID)
                    .version(config::VERSION)
                    .copyright(gettext("© 2025 The MQTTy Authors"))
                    .developer_name(gettext("The MQTTy Authors"))
                    .translator_credits(gettext("translator-credits"))
                    .license_type(gtk::License::Gpl30)
                    .issue_url("https://github.com/otaxhu/MQTTy/issues")
                    .website("https://github.com/otaxhu/MQTTy")
                    .build();

                about_dialog.present(app.active_window().as_ref());
            })
            .build();

        self.add_action_entries([action_quit, action_about]);
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Control>q"]);
    }

    fn setup_css(&self) {
        // // Libadwaita automatically reads the style.css file for us.
        //
        // let provider = gtk::CssProvider::new();
        // provider.load_from_resource("/io/github/otaxhu/MQTTy/style.css");
        // if let Some(display) = gdk::Display::default() {
        //     gtk::style_context_add_provider_for_display(
        //         &display,
        //         &provider,
        //         gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        //     );
        // }
    }
}
