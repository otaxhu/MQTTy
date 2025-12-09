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

use std::borrow::Cow;
use std::ffi::CStr;

#[cfg(target_os = "windows")]
use gtk::glib;
use icu::calendar::Gregorian as IcuGregorian;
use icu::datetime::fieldsets::YMDT as IcuYMDT;
use icu::datetime::DateTimeFormatter as IcuDateTimeFormatter;
use icu::locale::locale as icu_locale;
use icu::time::DateTime as IcuDateTime;

/// Result of this function can be unwrapped without problems, with the condition
/// met that `posix` must be a valid POSIX locale.
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

fn get_icu_date_time_formatter() -> IcuDateTimeFormatter<IcuYMDT> {
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
        //
        // Note:
        //
        // Current translations may be different from current locale, e.g.
        // If current locale is "pt_BR", gettext may load "pt" translations. We prefer
        // to format dates in current locale rather than current translations, so this
        // is perfectly fine.
        let current_locale = getlocale().expect("current locale could not be determined");

        // FIXME: handle "C" minimal locale case?? It should be impossible to happen
        // since a language was loaded by gettext

        Cow::Owned(current_locale)
    };

    let lang = locale_posix_to_bcp47(&lang).unwrap();

    let locale: icu::locale::Locale = lang.parse().unwrap();

    let ymdt = IcuYMDT::medium();

    IcuDateTimeFormatter::try_new(locale.into(), ymdt)
        // Fallback to "en" if ICU doesn't support formatting in current locale
        .or_else(|_| IcuDateTimeFormatter::try_new(icu_locale!("en").into(), ymdt))
        .unwrap()
}

/// Formats `iso_datetime` depending on current locale, it fallbacks to format in
/// "en" locale if any of this conditions are met:
///
/// 1. No translations loaded for current locale.
/// 2. Or, current locale is not supported by ICU.
///
/// Returns `None` if `iso_datetime` is not a valid ISO 8601 datetime.
pub fn format_datetime(iso_datetime: &str) -> Option<String> {
    let date_fmt = get_icu_date_time_formatter();

    let icu_datetime = IcuDateTime::try_from_str(iso_datetime, IcuGregorian).ok()?;

    Some(date_fmt.format(&icu_datetime).to_string())
}
