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

use crate::subclass::prelude::*;
use crate::widgets::MQTTyBaseCard;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/add_conn_card.ui")]
    pub struct MQTTyAddConnCard {}

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyAddConnCard {
        const NAME: &'static str = "MQTTyAddConnCard";

        type Type = super::MQTTyAddConnCard;

        type ParentType = MQTTyBaseCard;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MQTTyAddConnCard {}
    impl WidgetImpl for MQTTyAddConnCard {}
    impl FlowBoxChildImpl for MQTTyAddConnCard {}
    impl MQTTyBaseCardImpl for MQTTyAddConnCard {}
}

glib::wrapper! {
    pub struct MQTTyAddConnCard(ObjectSubclass<imp::MQTTyAddConnCard>)
        @extends gtk::Widget, gtk::FlowBoxChild, MQTTyBaseCard,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTyAddConnCard {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}
