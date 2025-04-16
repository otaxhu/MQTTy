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
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/publish_view/publish_body_tab.ui")]
    #[properties(wrapper_type = super::MQTTyPublishBodyTab)]
    pub struct MQTTyPublishBodyTab {
        #[property(get, set, override_interface = MQTTyDisplayModeIface)]
        display_mode: Cell<MQTTyDisplayMode>,
    }

    impl Default for MQTTyPublishBodyTab {
        fn default() -> Self {
            Self {
                display_mode: Cell::new(MQTTyDisplayMode::Desktop),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyPublishBodyTab {
        const NAME: &'static str = "MQTTyPublishBodyTab";

        type Type = super::MQTTyPublishBodyTab;

        type ParentType = adw::Bin;

        type Interfaces = (MQTTyDisplayModeIface,);

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyPublishBodyTab {}
    impl WidgetImpl for MQTTyPublishBodyTab {}
    impl BinImpl for MQTTyPublishBodyTab {}

    impl MQTTyDisplayModeIfaceImpl for MQTTyPublishBodyTab {}

    #[gtk::template_callbacks]
    impl MQTTyPublishBodyTab {
        #[template_callback]
        fn display_mode_to_orientation(&self, display_mode: MQTTyDisplayMode) -> gtk::Orientation {
            match display_mode {
                MQTTyDisplayMode::Desktop => gtk::Orientation::Horizontal,
                MQTTyDisplayMode::Mobile => gtk::Orientation::Vertical,
            }
        }

        #[template_callback]
        fn display_mode_to_vscroll_policy(
            &self,
            display_mode: MQTTyDisplayMode,
        ) -> gtk::PolicyType {
            match display_mode {
                MQTTyDisplayMode::Desktop => gtk::PolicyType::Automatic,
                MQTTyDisplayMode::Mobile => gtk::PolicyType::Never,
            }
        }
    }
}

glib::wrapper! {
    pub struct MQTTyPublishBodyTab(ObjectSubclass<imp::MQTTyPublishBodyTab>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
