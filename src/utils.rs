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
use gtk::{gio, glib};

use crate::client::{MQTTyClientQos, MQTTyClientVersion};

pub fn connect_mqtt_version_action(
    widget: &impl IsA<gtk::Widget>,
    group: &gio::SimpleActionGroup,
) -> gio::Action {
    let mqtt_version_state = gio::SimpleAction::new_stateful(
        "mqtt-version",
        Some(glib::VariantTy::STRING),
        &"v3".into(),
    );
    mqtt_version_state
        .bind_property("state", &*widget, "mqtt_version")
        .bidirectional()
        .sync_create()
        .transform_to(|_, state: glib::Variant| {
            let version = match state.str().unwrap() {
                "v3" => MQTTyClientVersion::V3X,
                "v5" => MQTTyClientVersion::V5,
                version => panic!("invalid MQTT version: {version}"),
            };

            Some(version)
        })
        .transform_from(|_, mqtt_version: MQTTyClientVersion| {
            let new_state = match mqtt_version {
                MQTTyClientVersion::V3X => "v3",
                MQTTyClientVersion::V5 => "v5",
            };

            Some(glib::Variant::from(new_state))
        })
        .build();

    group.add_action(&mqtt_version_state);

    mqtt_version_state.upcast()
}

pub fn connect_qos_action(
    widget: &impl IsA<gtk::Widget>,
    group: &gio::SimpleActionGroup,
) -> gio::Action {
    let qos_state =
        gio::SimpleAction::new_stateful("qos", Some(glib::VariantTy::STRING), &"qos_0".into());
    qos_state
        .bind_property("state", &*widget, "qos")
        .bidirectional()
        .sync_create()
        .transform_to(|_, state: glib::Variant| {
            let qos = match state.str().unwrap() {
                "qos_0" => MQTTyClientQos::Qos0,
                "qos_1" => MQTTyClientQos::Qos1,
                "qos_2" => MQTTyClientQos::Qos2,
                qos => panic!("invalid MQTT QoS: {qos}"),
            };

            Some(qos)
        })
        .transform_from(|_, qos: MQTTyClientQos| {
            let new_state = match qos {
                MQTTyClientQos::Qos0 => "qos_0",
                MQTTyClientQos::Qos1 => "qos_1",
                MQTTyClientQos::Qos2 => "qos_2",
            };

            Some(glib::Variant::from(new_state))
        })
        .build();

    group.add_action(&qos_state);

    qos_state.upcast()
}

/// Important:
///
/// You need to call this to update your fg accent color when adw::StyleManager
/// notifies ":accent-color" and ":dark" property changes
///
/// This fg accent color is dark-mode aware.
pub fn get_fg_accent_color_as_hex() -> &'static str {
    let man = adw::StyleManager::default();

    let color = if man.is_system_supports_accent_colors() {
        man.accent_color()
    } else {
        // Pretty nice looking color to use as fallback, it also mostly matches the MQTT
        // branding colors
        adw::AccentColor::Purple
    };

    let is_dark = man.is_dark();

    // See: https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/css-variables.html#accent-colors
    let (light_color, dark_color) = match color {
        adw::AccentColor::Blue => ("#0461be", "#81d0ff"),
        adw::AccentColor::Teal => ("#007184", "#7bdff4"),
        adw::AccentColor::Green => ("#15772e", "#8de698"),
        adw::AccentColor::Yellow => ("#905300", "#ffc057"),
        adw::AccentColor::Orange => ("#b62200", "#ff9c5b"),
        adw::AccentColor::Red => ("#c00023", "#ff888c"),
        adw::AccentColor::Pink => ("#a2326c", "#ffa0d8"),
        adw::AccentColor::Purple => ("#8939a4", "#fba7ff"),
        adw::AccentColor::Slate => ("#526678", "#bbd1e5"),
        c => panic!("Invalid color: {c:?}"),
    };

    if is_dark {
        dark_color
    } else {
        light_color
    }
}

/// An exact copy of GTK's private function of the same name, to be used on Adw.HeaderBar's
/// claiming gesture events logic for this application.
///
/// See: https://gitlab.gnome.org/GNOME/gtk/-/blob/main/gtk/gtkdragsource.c#L832
pub fn gtk_drag_check_threshold_double(
    widget: &impl IsA<gtk::Widget>,
    start_point: (f64, f64),
    offset_point: (f64, f64),
) -> bool {
    let drag_threshold = widget.settings().gtk_dnd_drag_threshold();

    (offset_point.0 - start_point.0).abs() > drag_threshold as f64
        || (offset_point.1 - start_point.1).abs() > drag_threshold as f64
}
