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

mod application;
mod client;
#[rustfmt::skip]
mod config;
mod display_mode;
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
