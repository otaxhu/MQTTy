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

use std::{borrow::Cow, ffi::CStr};

use adw::prelude::*;
use gtk::{gio, glib};
use icu::datetime::DateTimeFormatter;

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
/// libadwaita must be initialized before calling this function
pub fn get_accent_color_as_hex() -> &'static str {
    let man = adw::StyleManager::default();

    let color = if man.is_system_supports_accent_colors() {
        man.accent_color()
    } else {
        // Pretty nice looking color to use as fallback, it also mostly matches the MQTT
        // branding colors
        adw::AccentColor::Purple
    };

    // See: https://gnome.pages.gitlab.gnome.org/libadwaita/doc/1-latest/enum.AccentColor.html
    match color {
        adw::AccentColor::Blue => "#3584e4",
        adw::AccentColor::Teal => "#2190a4",
        adw::AccentColor::Green => "#3a944a",
        adw::AccentColor::Yellow => "#c88800",
        adw::AccentColor::Orange => "#ed5b00",
        adw::AccentColor::Red => "#e62d42",
        adw::AccentColor::Pink => "#d56199",
        adw::AccentColor::Purple => "#9141ac",
        adw::AccentColor::Slate => "#6f8396",
        c => panic!("Invalid color: {c:?}"),
    }
}

// Localization-related functions

fn locale_posix_to_bcp47(posix: &str) -> Option<Cow<'_, str>> {
    let main_part = posix.split('.').next()?.split('@').next()?;

    let mut parts = main_part.split('_');
    let lang = parts.next()?;
    let region = parts.next();

    Some(match region {
        Some(r) => Cow::Owned(format!("{lang}-{r}")),
        None => Cow::Borrowed(lang),
    })
}

#[cfg(target_os = "windows")]
fn getlocale() -> Option<String> {
    // We need the POSIX form, so we are calling this instead of the MS setlocale one
    //
    // See:
    // https://gitlab.gnome.org/GNOME/glib/-/blob/main/glib/gwin32.c#L94
    Some(glib::win32_getlocale().into())
}

#[cfg(not(target_os = "windows"))]
fn getlocale() -> Option<String> {
    // SAFETY:
    //
    // setlocale returned string is read-only and owned by the libc, here we are cloning it
    // if it's not NULL, so it is safe.
    //
    // setlocale is also marked as MT-Unsafe (in Linux), but I think we are not
    // calling from a different thread than the main one.
    //
    // POSIX Reference:
    //
    // > The application shall not modify the string returned which may be
    // > overwritten by a subsequent call to setlocale().
    //
    // https://pubs.opengroup.org/onlinepubs/009695399/functions/setlocale.html
    unsafe {
        let res = gettext_sys::setlocale(gettextrs::LocaleCategory::LcAll as i32, std::ptr::null());
        if res.is_null() {
            None
        } else {
            CStr::from_ptr(res).to_str().ok().map(|s| s.to_owned())
        }
    }
}

pub fn get_icu_date_time_formatter() -> DateTimeFormatter<icu::datetime::fieldsets::YMDT> {
    // We rename so that xgettext command doesn't try to translate the call we
    // are doing below
    use gettextrs::gettext as _private_gettext;

    let header = _private_gettext("");

    // Following almost the same logic from:
    //
    // https://gitlab.gnome.org/GNOME/glib/-/blob/main/glib/ggettext.c#L313
    let lang = if header.is_empty() {
        // If it's empty, that means that no language was loaded for the current locale.
        // We default to "en"
        Cow::Borrowed("en")
    } else {
        // If it's not empty, a language was loaded for the current locale, we query it.
        let current_locale = getlocale().expect("current locale could not be determined");

        // FIXME: handle "C" minimal locale case?? It should be impossible to happen
        // since a language was loaded by gettext

        Cow::Owned(current_locale)
    };

    let lang = locale_posix_to_bcp47(&lang).unwrap();

    let locale: icu::locale::Locale = lang.parse().unwrap();

    DateTimeFormatter::try_new(locale.into(), icu::datetime::fieldsets::YMDT::medium()).unwrap()
}
