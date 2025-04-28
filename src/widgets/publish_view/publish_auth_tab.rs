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
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/publish_view/publish_auth_tab.ui")]
    #[properties(wrapper_type = super::MQTTyPublishAuthTab)]
    pub struct MQTTyPublishAuthTab {
        #[property(get, set)]
        username: RefCell<String>,

        #[property(get, set)]
        password: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyPublishAuthTab {
        const NAME: &'static str = "MQTTyPublishAuthTab";

        type Type = super::MQTTyPublishAuthTab;

        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyPublishAuthTab {}
    impl WidgetImpl for MQTTyPublishAuthTab {}
    impl BinImpl for MQTTyPublishAuthTab {}
}

glib::wrapper! {
    pub struct MQTTyPublishAuthTab(ObjectSubclass<imp::MQTTyPublishAuthTab>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
