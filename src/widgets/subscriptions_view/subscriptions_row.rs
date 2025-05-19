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

use crate::client::{MQTTyClientQos, MQTTyClientVersion};
use crate::utils;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/subscriptions_row.ui")]
    #[properties(wrapper_type = super::MQTTySubscriptionsRow)]
    pub struct MQTTySubscriptionsRow {
        #[property(get, set)]
        subtitle: RefCell<String>,

        #[property(get, set)]
        prefix_widget: RefCell<Option<gtk::Widget>>,

        #[property(get, set)]
        url: RefCell<String>,

        #[property(get, set)]
        topic: RefCell<String>,

        #[property(get, set)]
        editing: Cell<bool>,

        #[property(get, set, builder(Default::default()))]
        mqtt_version: Cell<MQTTyClientVersion>,

        #[property(get, set, builder(Default::default()))]
        qos: Cell<MQTTyClientQos>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionsRow {
        const NAME: &'static str = "MQTTySubscriptionsRow";

        type Type = super::MQTTySubscriptionsRow;

        type ParentType = adw::PreferencesRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTySubscriptionsRow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            utils::connect_mqtt_version_and_qos_actions(&*obj, "subscriptions-row");

            // A little bit of Libadwaita CSS witchcraft in order for this object
            // to behave like an expander row
            //
            // See:
            //
            // https://gitlab.gnome.org/GNOME/libadwaita/-/blob/main/src/stylesheet/widgets/_lists.scss
            obj.connect_editing_notify(|obj| {
                if obj.editing() {
                    obj.set_state_flags(gtk::StateFlags::CHECKED, false);
                } else {
                    obj.unset_state_flags(gtk::StateFlags::CHECKED);
                }
            });
        }
    }
    impl WidgetImpl for MQTTySubscriptionsRow {}
    impl ListBoxRowImpl for MQTTySubscriptionsRow {}
    impl PreferencesRowImpl for MQTTySubscriptionsRow {}

    #[gtk::template_callbacks]
    impl MQTTySubscriptionsRow {
        #[template_callback]
        fn on_row_activated(&self) {
            let obj = self.obj();

            let Ok(list_box) = obj.parent().unwrap().downcast::<gtk::ListBox>() else {
                return;
            };

            list_box.emit_by_name::<()>("row-activated", &[&*obj]);
        }

        #[template_callback]
        fn on_keynav_failed(&self, direction: gtk::DirectionType) -> bool {
            let new_direction = match direction {
                gtk::DirectionType::Up => gtk::DirectionType::TabBackward,
                gtk::DirectionType::Down => gtk::DirectionType::TabForward,
                _ => return false,
            };
            self.obj()
                .root()
                .map(|top_window| top_window.child_focus(new_direction))
                .unwrap_or(false)
        }
    }
}

glib::wrapper! {
    /// This is a modified version of Adw.ExpanderRow
    pub struct MQTTySubscriptionsRow(ObjectSubclass<imp::MQTTySubscriptionsRow>)
        @extends gtk::ListBoxRow, gtk::Widget, adw::PreferencesRow,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Actionable;
}

impl MQTTySubscriptionsRow {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}
