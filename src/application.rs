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

use std::cell::OnceCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{gio, glib};

use crate::config;
use crate::display_mode::{MQTTyDisplayMode, MQTTyDisplayModeIface};
use crate::main_window::MQTTyWindow;
use crate::widgets::{
    MQTTyKeyValueRow, MQTTyPublishAuthTab, MQTTyPublishBodyTab, MQTTyPublishGeneralTab,
    MQTTyPublishUserPropsTab, MQTTyPublishView, MQTTySourceView, MQTTySubscriptionDialog,
    MQTTySubscriptionMessagesSheet, MQTTySubscriptionRow, MQTTySubscriptionsConnectionDialog,
    MQTTySubscriptionsConnectionRow, MQTTySubscriptionsOverview, MQTTySubscriptionsView,
};

mod imp {

    use super::*;

    #[derive(Default)]
    pub struct MQTTyApplication {
        pub settings: OnceCell<gio::Settings>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyApplication {
        const NAME: &'static str = "MQTTyApplication";
        type Type = super::MQTTyApplication;
        type ParentType = adw::Application;

        fn class_init(_klass: &mut Self::Class) {
            // Eagerly initialize everything

            MQTTyWindow::static_type();

            // Widgets
            MQTTySourceView::static_type();
            MQTTyKeyValueRow::static_type();

            MQTTyPublishView::static_type();
            MQTTyPublishGeneralTab::static_type();
            MQTTyPublishBodyTab::static_type();
            MQTTyPublishUserPropsTab::static_type();
            MQTTyPublishAuthTab::static_type();

            MQTTySubscriptionsView::static_type();
            MQTTySubscriptionDialog::static_type();
            MQTTySubscriptionRow::static_type();
            MQTTySubscriptionMessagesSheet::static_type();
            MQTTySubscriptionsOverview::static_type();
            MQTTySubscriptionsConnectionRow::static_type();
            MQTTySubscriptionsConnectionDialog::static_type();

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

            app.setup_gactions();
            app.setup_accels();
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
                let about_dialog = adw::AboutDialog::from_appdata(
                    &format!("/io/github/otaxhu/MQTTy/{}.metainfo.xml", config::APP_ID),
                    Some(config::VERSION),
                );
                // Translators: See: https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/method.AboutDialog.set_translator_credits.html
                about_dialog.set_translator_credits(&gettext("translator-credits"));
                about_dialog.set_copyright(&gettext("© 2025 Oscar Pernia"));
                about_dialog.add_link(
                    &gettext("Help us translate"),
                    "https://hosted.weblate.org/engage/MQTTy/",
                );

                about_dialog.present(app.active_window().as_ref());
            })
            .build();

        self.add_action_entries([action_quit, action_about]);
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Control>q"]);

        self.set_accels_for_action("win.publish-send", &["<Control>Return"]);
        self.set_accels_for_action("win.publish-new-tab", &["<Control>t"]);
        self.set_accels_for_action("win.publish-delete-tab", &["<Control>w"]);

        self.set_accels_for_action("win.set-publish-view", &["<Alt>Left"]);
        self.set_accels_for_action("win.set-subscriptions-view", &["<Alt>Right"]);
    }
}
