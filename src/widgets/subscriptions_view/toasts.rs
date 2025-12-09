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

use adw::prelude::*;
use gettextrs::gettext;

use crate::application::MQTTyApplication;
use crate::main_window::MQTTyWindow;
use crate::toast::MQTTyToastBuilder;

/// Show this toast when the user is trying add a new client, but it already exists
/// in the controller.
pub fn client_already_exists() {
    let app = MQTTyApplication::get_singleton();

    let window = app.active_window().and_downcast::<MQTTyWindow>().unwrap();

    window.toast(
        MQTTyToastBuilder::new()
            .icon(
                gtk::Image::builder()
                    .icon_name("action-unavailable-symbolic")
                    .css_classes(["error"])
                    .build()
                    .as_ref(),
            )
            .title(gettext(
                "A client already exists with the same URL and/or client ID",
            ))
            .build()
            .as_ref(),
    );
}

/// Show this toast when the user is trying to add a new subscription to a client,
/// but the topic filter already exists in the client.
pub fn subscription_already_exists() {
    let app = MQTTyApplication::get_singleton();

    let window = app.active_window().and_downcast::<MQTTyWindow>().unwrap();

    window.toast(
        MQTTyToastBuilder::new()
            .icon(
                gtk::Image::builder()
                    .icon_name("action-unavailable-symbolic")
                    .css_classes(["error"])
                    .build()
                    .as_ref(),
            )
            .title(gettext(
                "This client already contains a subscription with the same topic filter",
            ))
            .build()
            .as_ref(),
    );
}
