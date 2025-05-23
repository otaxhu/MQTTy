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
use std::sync::LazyLock;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib::subclass::Signal;
use gtk::{gio, glib};

use crate::display_mode::{MQTTyDisplayMode, MQTTyDisplayModeIface};
use crate::objects::MQTTyKeyValue;
use crate::subclass::prelude::*;

mod imp {

    use super::*;

    #[derive(gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/key_value_row.ui")]
    #[properties(wrapper_type = super::MQTTyKeyValueRow)]
    pub struct MQTTyKeyValueRow {
        #[property(get, set)]
        active: Cell<bool>,

        #[property(get, set, override_interface = MQTTyDisplayModeIface)]
        display_mode: Cell<MQTTyDisplayMode>,

        #[property(get, set)]
        key: RefCell<String>,

        #[property(get, set)]
        value: RefCell<String>,

        #[property(get, set)]
        user_changed: Cell<bool>,
    }

    impl Default for MQTTyKeyValueRow {
        fn default() -> Self {
            Self {
                display_mode: Cell::new(MQTTyDisplayMode::Desktop),
                active: Default::default(),
                key: Default::default(),
                value: Default::default(),
                user_changed: Default::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyKeyValueRow {
        const NAME: &'static str = "MQTTyKeyValueRow";

        type Type = super::MQTTyKeyValueRow;

        type ParentType = gtk::ListBoxRow;

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
    impl ObjectImpl for MQTTyKeyValueRow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let changed_callback = |obj: &Self::Type| {
                obj.emit_by_name::<()>("changed", &[]);
            };

            obj.connect_key_notify(changed_callback);
            obj.connect_value_notify(changed_callback);

            let delete_action = gio::SimpleAction::new("delete", None);

            obj.bind_property("user_changed", &delete_action, "enabled")
                .sync_create()
                .build();

            delete_action.connect_activate(glib::clone!(
                #[weak]
                obj,
                move |_, _| {
                    obj.emit_by_name::<()>("deleted", &[]);
                }
            ));

            let group = gio::SimpleActionGroup::new();
            group.add_action(&delete_action);

            obj.insert_action_group("row", Some(&group));
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
                vec![
                    Signal::builder("deleted").build(),
                    Signal::builder("changed").build(),
                ]
            });
            &*SIGNALS
        }
    }
    impl WidgetImpl for MQTTyKeyValueRow {}
    impl ListBoxRowImpl for MQTTyKeyValueRow {}
    impl MQTTyDisplayModeIfaceImpl for MQTTyKeyValueRow {}

    #[gtk::template_callbacks]
    impl MQTTyKeyValueRow {
        #[template_callback]
        fn display_mode_to_orientation(&self, display_mode: MQTTyDisplayMode) -> gtk::Orientation {
            match display_mode {
                MQTTyDisplayMode::Desktop => gtk::Orientation::Horizontal,
                MQTTyDisplayMode::Mobile => gtk::Orientation::Vertical,
            }
        }
    }
}

glib::wrapper! {
    /// Emits "deleted" when delete button is pressed, it doesn't delete anything, its meant
    /// that upper layers connect to it and handle it
    ///
    /// Emits "changed" when key or value props are changed
    pub struct MQTTyKeyValueRow(ObjectSubclass<imp::MQTTyKeyValueRow>)
        @extends gtk::Widget, gtk::ListBoxRow,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Actionable;
}

impl Default for MQTTyKeyValueRow {
    fn default() -> Self {
        Self::new(false, false, "", "")
    }
}

impl MQTTyKeyValueRow {
    pub fn new(active: bool, user_changed: bool, key: &str, value: &str) -> Self {
        glib::Object::builder()
            .property("active", active)
            .property("user_changed", user_changed)
            .property("key", key)
            .property("value", value)
            .build()
    }
}

impl From<MQTTyKeyValue> for MQTTyKeyValueRow {
    fn from(value: MQTTyKeyValue) -> Self {
        Self::new(
            value.active(),
            Default::default(),
            &value.key(),
            &value.value(),
        )
    }
}
