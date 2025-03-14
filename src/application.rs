use std::cell::OnceCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{gio, glib};

use crate::config;
use crate::main_window::MQTTyWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct MQTTyApplication {
        pub settings: OnceCell<gio::Settings>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyApplication {
        const NAME: &'static str = "MQTTyApplication";
        type Type = super::MQTTyApplication;
        type ParentType = adw::Application;
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
            let app = self.obj();

            // Set icons for shell
            gtk::Window::set_default_icon_name(config::APP_ID);

            app.setup_css();
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
