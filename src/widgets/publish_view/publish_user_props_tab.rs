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

use std::cell::Cell;

use adw::subclass::prelude::*;
use gtk::glib;

use crate::display_mode::{MQTTyDisplayMode, MQTTyDisplayModeIface};
use crate::subclass::prelude::*;

mod imp {

    use super::*;

    #[derive(gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/publish_view/publish_user_props_tab.ui")]
    #[properties(wrapper_type = super::MQTTyPublishUserPropsTab)]
    pub struct MQTTyPublishUserPropsTab {
        #[property(get, set, override_interface = MQTTyDisplayModeIface)]
        display_mode: Cell<MQTTyDisplayMode>,
    }

    impl Default for MQTTyPublishUserPropsTab {
        fn default() -> Self {
            Self {
                display_mode: Cell::new(MQTTyDisplayMode::Desktop),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyPublishUserPropsTab {
        const NAME: &'static str = "MQTTyPublishUserPropsTab";

        type Type = super::MQTTyPublishUserPropsTab;

        type ParentType = adw::Bin;

        type Interfaces = (MQTTyDisplayModeIface,);

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyPublishUserPropsTab {}
    impl WidgetImpl for MQTTyPublishUserPropsTab {}
    impl BinImpl for MQTTyPublishUserPropsTab {}

    impl MQTTyDisplayModeIfaceImpl for MQTTyPublishUserPropsTab {}
}

glib::wrapper! {
    pub struct MQTTyPublishUserPropsTab(ObjectSubclass<imp::MQTTyPublishUserPropsTab>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
