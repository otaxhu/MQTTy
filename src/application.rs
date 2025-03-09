use std::cell::OnceCell;

use adw::subclass::prelude::*;
use gettextrs::gettext;
use glib::WeakRef;
use gtk::prelude::*;
use gtk::{gio, glib};
use tracing::{debug, info};

use crate::config::{APP_ID, PKGDATADIR, PROFILE, VERSION};
use crate::main_window::MQTTyWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct MQTTyApplication {
        pub window: OnceCell<WeakRef<MQTTyWindow>>,
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
            debug!("GtkApplication<ExampleApplication>::activate");
            self.parent_activate();
            let app = self.obj();

            if let Some(window) = self.window.get() {
                let window = window.upgrade().unwrap();
                window.present();
                return;
            }

            let window = MQTTyWindow::new(&app);
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            app.main_window().present();
        }

        fn startup(&self) {
            debug!("GtkApplication<ExampleApplication>::startup");
            self.parent_startup();
            let app = self.obj();

            // Set icons for shell
            gtk::Window::set_default_icon_name(APP_ID);

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
    fn main_window(&self) -> MQTTyWindow {
        self.imp().window.get().unwrap().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        // Quit
        let action_quit = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| {
                // This is needed to trigger the delete event and saving the window state
                app.main_window().close();
                app.quit();
            })
            .build();

        // About
        let action_about = gio::ActionEntry::builder("about")
            .activate(|app: &Self, _, _| {
                app.show_about_dialog();
            })
            .build();
        self.add_action_entries([action_quit, action_about]);
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Control>q"]);
        self.set_accels_for_action("window.close", &["<Control>w"]);
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

    fn show_about_dialog(&self) {
        let dialog = gtk::AboutDialog::builder()
            .logo_icon_name(APP_ID)
            // Insert your license of choice here
            .license_type(gtk::License::Gpl30)
            // Insert your website here
            // .website("https://gitlab.gnome.org/bilelmoussaoui/MQTTy/")
            .version(VERSION)
            .transient_for(&self.main_window())
            .translator_credits(gettext("translator-credits"))
            .modal(true)
            .authors(vec!["Oscar Pernia"])
            .artists(vec!["Oscar Pernia"])
            .build();

        dialog.present();
    }

    pub fn run(&self) -> glib::ExitCode {
        info!("MQTTy ({})", APP_ID);
        info!("Version: {} ({})", VERSION, PROFILE);
        info!("Datadir: {}", PKGDATADIR);

        ApplicationExtManual::run(self)
    }
}

impl Default for MQTTyApplication {
    fn default() -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .property("resource-base-path", "/io/github/otaxhu/MQTTy/")
            .build()
    }
}
