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
use glib::translate::*;
use gtk::glib;

#[repr(C)]
pub struct _AdwIndicatorBin {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

glib::wrapper! {
    /// FIXME: If this becomes public API, remove this file and use that instead.
    /// See: https://gitlab.gnome.org/GNOME/libadwaita/-/issues/1107
    ///
    /// Make sure that ":child" is a gtk::Image which has an icon set in ":icon-name",
    /// if you fail to do so you may have rendering issues.
    /// See: https://gitlab.gnome.org/GNOME/libadwaita/-/issues/1107#note_2631128
    pub struct AdwIndicatorBin(Object<_AdwIndicatorBin>) @extends gtk::Widget;
    match fn {
        type_ => || glib::Type::from_name("AdwIndicatorBin").unwrap().into_glib(),
    }
}

impl AdwIndicatorBin {
    pub fn new() -> Self {
        let o: Self = glib::Object::new();
        o.connect_notify_local(Some("child"), |o, _| match o.child() {
            Some(c) => {
                if !c.is::<gtk::Image>() {
                    glib::g_warning!("MQTTy", "indicator-bin: child is not a GtkImage");
                }
            }
            _ => {}
        });
        o
    }

    pub fn child(&self) -> Option<gtk::Widget> {
        self.property("child")
    }

    pub fn set_child(&self, child: Option<&impl IsA<gtk::Widget>>) {
        self.set_property("child", child);
    }

    // Cannot implement this getter succesfully, libadwaita >=1.9 added :badge-number
    // prop, but previous versions don't have that and instead used a plain string
    // that cannot be converted back to the badge number
    /*
    pub fn badge_number(&self) -> u32 { ... }
    */

    pub fn set_badge_number(&self, badge_number: u32) {
        if self.has_property("badge-number") {
            // Compatibility logic for libadwaita >=1.9
            self.set_property("badge-number", badge_number);
        } else {
            // Compatibility logic for libadwaita <1.9
            let badge = match badge_number {
                0 => "".to_string(),
                (1000..) => format!("999+"),
                n => format!("{n}"),
            };
            self.set_property("badge", badge);
        }
    }

    pub fn needs_attention(&self) -> bool {
        self.property("needs-attention")
    }

    pub fn set_needs_attention(&self, needs_attention: bool) {
        self.set_property("needs-attention", needs_attention);
    }
}
