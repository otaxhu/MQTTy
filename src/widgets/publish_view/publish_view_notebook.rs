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

use std::cell::{Cell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use crate::display_mode::{MQTTyDisplayMode, MQTTyDisplayModeIface};
use crate::subclass::prelude::*;

mod imp {

    use super::*;

    #[derive(gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/publish_view/publish_view_notebook.ui")]
    #[properties(wrapper_type = super::MQTTyPublishViewNotebook)]
    pub struct MQTTyPublishViewNotebook {
        #[property(get, set, override_interface = MQTTyDisplayModeIface)]
        display_mode: Cell<MQTTyDisplayMode>,

        #[property(get, set)]
        mqtt_version: RefCell<String>,

        #[property(get, set)]
        topic: RefCell<String>,

        #[property(get, set)]
        url: RefCell<String>,

        #[property(get, set)]
        qos: RefCell<String>,
    }

    impl Default for MQTTyPublishViewNotebook {
        fn default() -> Self {
            Self {
                display_mode: Cell::new(MQTTyDisplayMode::Desktop),
                mqtt_version: Default::default(),
                topic: Default::default(),
                url: Default::default(),
                qos: Default::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyPublishViewNotebook {
        const NAME: &'static str = "MQTTyPublishViewNotebook";

        type Type = super::MQTTyPublishViewNotebook;

        type ParentType = adw::Bin;

        type Interfaces = (MQTTyDisplayModeIface,);

        fn class_init(klass: &mut Self::Class) {
            klass.install_property_action("publish-view-notebook.mqtt-version", "mqtt_version");
            klass.install_property_action("publish-view-notebook.qos", "qos");
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyPublishViewNotebook {}
    impl WidgetImpl for MQTTyPublishViewNotebook {}
    impl BinImpl for MQTTyPublishViewNotebook {}
    impl MQTTyDisplayModeIfaceImpl for MQTTyPublishViewNotebook {}
}

glib::wrapper! {
    pub struct MQTTyPublishViewNotebook(ObjectSubclass<imp::MQTTyPublishViewNotebook>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTyPublishViewNotebook {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}
