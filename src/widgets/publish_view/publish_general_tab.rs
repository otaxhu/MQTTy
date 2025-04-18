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

use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/publish_view/publish_general_tab.ui")]
    #[properties(wrapper_type = super::MQTTyPublishGeneralTab)]
    pub struct MQTTyPublishGeneralTab {
        #[property(get, set)]
        topic: RefCell<String>,

        #[property(get, set)]
        url: RefCell<String>,

        #[template_child]
        mqtt_3_button: TemplateChild<gtk::CheckButton>,
        #[template_child]
        mqtt_5_button: TemplateChild<gtk::CheckButton>,

        #[template_child]
        qos_0_button: TemplateChild<gtk::CheckButton>,
        #[template_child]
        qos_1_button: TemplateChild<gtk::CheckButton>,
        #[template_child]
        qos_2_button: TemplateChild<gtk::CheckButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyPublishGeneralTab {
        const NAME: &'static str = "MQTTyPublishGeneralTab";

        type Type = super::MQTTyPublishGeneralTab;

        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyPublishGeneralTab {
        fn constructed(&self) {
            self.parent_constructed();

            self.mqtt_3_button.set_action_target(Some("3"));
            self.mqtt_5_button.set_action_target(Some("5"));

            self.qos_0_button.set_action_target(Some("0"));
            self.qos_1_button.set_action_target(Some("1"));
            self.qos_2_button.set_action_target(Some("2"));
        }
    }
    impl WidgetImpl for MQTTyPublishGeneralTab {}
    impl BinImpl for MQTTyPublishGeneralTab {}

    #[gtk::template_callbacks]
    impl MQTTyPublishGeneralTab {
        #[template_callback]
        fn or(&self, a: bool, b: bool) -> bool {
            a || b
        }
    }
}

glib::wrapper! {
    pub struct MQTTyPublishGeneralTab(ObjectSubclass<imp::MQTTyPublishGeneralTab>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
