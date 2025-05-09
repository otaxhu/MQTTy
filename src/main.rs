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

#![windows_subsystem = "windows"]

mod application;
mod client;
#[rustfmt::skip]
mod config;
mod content_type;
mod display_mode;
mod gsettings;
mod main_window;
mod objects;
mod pages;
mod subclass;
mod toast;
mod widgets;

use std::path::PathBuf;

use gettextrs::LocaleCategory;
use gtk::prelude::*;
use gtk::{gio, glib};

use self::application::MQTTyApplication;
use self::config::GETTEXT_PACKAGE;

// Returns a path relative to the application root directory
//
// This function expects the application executable to be inside of a bin/ directory,
// which is inside of the application root directory
fn app_root_rel_path(path: &str) -> PathBuf {
    let root_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    root_dir.join(path)
}

fn main() -> glib::ExitCode {
    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, app_root_rel_path("share/locale"))
        .expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    glib::set_application_name("MQTTy");

    // Prepare XDG_DATA_DIRS env variable
    let datadir = app_root_rel_path("share");
    let xdg_data_dirs: Vec<PathBuf> = match std::env::var("XDG_DATA_DIRS") {
        Ok(dirs) => std::env::split_paths(&dirs).collect(),
        Err(_) => vec![],
    };
    if !xdg_data_dirs.iter().any(|d| d == &datadir) {
        let mut new_dirs = vec![datadir];
        new_dirs.extend(xdg_data_dirs);
        let xdg_data_dir = std::env::join_paths(&new_dirs).unwrap();
        std::env::set_var("XDG_DATA_DIRS", xdg_data_dir);
    }

    // Prepare GResources
    let res = gio::Resource::load(app_root_rel_path("share/MQTTy/MQTTy.gresource"))
        .expect("Could not load gresource file");
    gio::resources_register(&res);

    // // Libadwaita initializes on MQTTyApplication startup
    //
    // adw::init().unwrap();

    let app = MQTTyApplication::get_singleton();
    app.run()
}
