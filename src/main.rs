mod application;
#[rustfmt::skip]
mod config;
mod gsettings;
mod main_window;
mod pages;
mod subclass;
mod widgets;

use gettextrs::{gettext, LocaleCategory};
use gtk::prelude::*;
use gtk::{gio, glib};

use self::application::MQTTyApplication;
use self::config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE};

fn main() -> glib::ExitCode {
    // Initialize logger
    tracing_subscriber::fmt::init();

    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    glib::set_application_name(&gettext("MQTTy"));

    let res = gio::Resource::load(RESOURCES_FILE).expect("Could not load gresource file");
    gio::resources_register(&res);

    // // Libadwaita initializes on MQTTyApplication startup
    //
    // adw::init().unwrap();

    let app = MQTTyApplication::get_singleton();
    app.run()
}
