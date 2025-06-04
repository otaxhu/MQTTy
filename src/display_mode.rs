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

use adw::subclass::prelude::*;
use gtk::glib;
use gtk::glib::gobject_ffi;

#[derive(Default, Copy, Clone, glib::Enum, PartialEq)]
#[enum_type(name = "MQTTyDisplayMode")]
pub enum MQTTyDisplayMode {
    #[default]
    #[enum_value(nick = "desktop")]
    Desktop,
    #[enum_value(nick = "mobile")]
    Mobile,
}

mod imp_iface {

    use std::sync::LazyLock;

    use super::*;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct MQTTyDisplayModeIfaceC {
        iface: gobject_ffi::GTypeInterface,
    }

    unsafe impl InterfaceStruct for MQTTyDisplayModeIfaceC {
        type Type = MQTTyDisplayModeIface;
    }

    pub struct MQTTyDisplayModeIface {}

    #[glib::object_interface]
    impl ObjectInterface for MQTTyDisplayModeIface {
        const NAME: &'static str = "MQTTyDisplayModeIface";

        type Interface = MQTTyDisplayModeIfaceC;

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPS: LazyLock<Vec<glib::ParamSpec>> = LazyLock::new(|| {
                vec![
                    glib::ParamSpecEnum::builder_with_default::<MQTTyDisplayMode>(
                        "display-mode",
                        MQTTyDisplayMode::Desktop,
                    )
                    .build(),
                ]
            });
            &*PROPS
        }
    }
}

glib::wrapper! {
    /// Interface that just declares the :display-mode enum property to the objects that
    /// implements it.
    ///
    /// :display-mode property can be two values, "desktop" or "mobile", it hints the widget's
    /// styling/layout depending on which value it is.
    pub struct MQTTyDisplayModeIface(ObjectInterface<imp_iface::MQTTyDisplayModeIface>);
}

pub trait MQTTyDisplayModeIfaceImpl: ObjectImpl {}

unsafe impl<T: MQTTyDisplayModeIfaceImpl> IsImplementable<T> for MQTTyDisplayModeIface {}
