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
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/base_card.ui")]
    #[properties(wrapper_type = super::MQTTyBaseCard)]
    pub struct MQTTyBaseCard {
        #[property(get, set)]
        prefix_widget: RefCell<Option<gtk::Widget>>,

        #[property(get, set)]
        title: RefCell<String>,

        #[property(get, set)]
        subtitle: RefCell<String>,

        #[template_child]
        title_label: TemplateChild<gtk::Label>,

        #[template_child]
        subtitle_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyBaseCard {
        const NAME: &'static str = "MQTTyBaseCard";

        type Type = super::MQTTyBaseCard;

        type ParentType = gtk::FlowBoxChild;

        const ABSTRACT: bool = true;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyBaseCard {}
    impl WidgetImpl for MQTTyBaseCard {}
    impl FlowBoxChildImpl for MQTTyBaseCard {}
}

glib::wrapper! {
    pub struct MQTTyBaseCard(ObjectSubclass<imp::MQTTyBaseCard>)
        @extends gtk::Widget, gtk::FlowBoxChild,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

pub trait MQTTyBaseCardImpl: FlowBoxChildImpl {}

unsafe impl<T: MQTTyBaseCardImpl> IsSubclassable<T> for MQTTyBaseCard {}
