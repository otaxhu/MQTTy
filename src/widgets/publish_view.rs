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

mod publish_body_tab;
mod publish_general_tab;
mod publish_user_props_tab;

pub use publish_body_tab::MQTTyPublishBodyTab;
pub use publish_general_tab::MQTTyPublishGeneralTab;
pub use publish_user_props_tab::MQTTyPublishUserPropsTab;

use std::cell::{Cell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use crate::display_mode::{MQTTyDisplayMode, MQTTyDisplayModeIface};
use crate::subclass::prelude::*;

mod imp {

    use super::*;

    #[derive(gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/publish_view/publish_view.ui")]
    #[properties(wrapper_type = super::MQTTyPublishView)]
    pub struct MQTTyPublishView {
        #[property(get, set)]
        mqtt_version: RefCell<String>,

        #[property(get, set, override_interface = MQTTyDisplayModeIface)]
        display_mode: Cell<MQTTyDisplayMode>,
    }

    impl Default for MQTTyPublishView {
        fn default() -> Self {
            Self {
                display_mode: Cell::new(MQTTyDisplayMode::Desktop),
                mqtt_version: Default::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyPublishView {
        const NAME: &'static str = "MQTTyPublishView";

        type Type = super::MQTTyPublishView;

        type ParentType = adw::Bin;

        type Interfaces = (MQTTyDisplayModeIface,);

        fn class_init(klass: &mut Self::Class) {
            klass.install_property_action("publish-view.mqtt-version", "mqtt_version");
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyPublishView {}
    impl WidgetImpl for MQTTyPublishView {}
    impl BinImpl for MQTTyPublishView {}
    impl MQTTyDisplayModeIfaceImpl for MQTTyPublishView {}

    #[gtk::template_callbacks]
    impl MQTTyPublishView {
        #[template_callback]
        fn or(&self, a: bool, b: bool) -> bool {
            a || b
        }
    }
}

glib::wrapper! {
    pub struct MQTTyPublishView(ObjectSubclass<imp::MQTTyPublishView>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
