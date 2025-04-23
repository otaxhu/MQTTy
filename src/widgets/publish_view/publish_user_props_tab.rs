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

use std::cell::{Cell, OnceCell, RefCell};
use std::iter;
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::gio;
use gtk::glib;

use crate::display_mode::{MQTTyDisplayMode, MQTTyDisplayModeIface};
use crate::objects::MQTTyKeyValue;
use crate::subclass::prelude::*;
use crate::widgets::MQTTyKeyValueRow;

mod imp {

    use super::*;

    #[derive(gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/publish_view/publish_user_props_tab.ui")]
    #[properties(wrapper_type = super::MQTTyPublishUserPropsTab)]
    pub struct MQTTyPublishUserPropsTab {
        #[property(get, set, override_interface = MQTTyDisplayModeIface)]
        display_mode: Cell<MQTTyDisplayMode>,

        /// The type of the items are MQTTyKeyValueRow
        row_model: OnceCell<gio::ListStore>,

        #[template_child]
        list_box: TemplateChild<gtk::ListBox>,
    }

    impl Default for MQTTyPublishUserPropsTab {
        fn default() -> Self {
            Self {
                display_mode: Cell::new(MQTTyDisplayMode::Desktop),
                row_model: Default::default(),
                list_box: Default::default(),
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
    impl ObjectImpl for MQTTyPublishUserPropsTab {
        fn constructed(&self) {
            self.parent_constructed();

            self.row_model().append(&self.new_trigger_row());

            self.list_box.bind_model(Some(&self.row_model()), |i| {
                i.downcast_ref::<gtk::Widget>().unwrap().clone()
            });
        }
    }
    impl WidgetImpl for MQTTyPublishUserPropsTab {}
    impl BinImpl for MQTTyPublishUserPropsTab {}

    impl MQTTyDisplayModeIfaceImpl for MQTTyPublishUserPropsTab {}

    impl MQTTyPublishUserPropsTab {
        /// The type of the items are MQTTyKeyValueRow
        fn row_model(&self) -> gio::ListStore {
            self.row_model
                .get_or_init(|| gio::ListStore::new::<MQTTyKeyValueRow>())
                .clone()
        }

        /// Returns a new trigger row, that triggers a new addition to row_model with another
        /// trigger row, when this row is changed by the user
        ///
        /// After the row gets changed, it becomes a normal row (it does not trigger additions)
        fn new_trigger_row(&self) -> MQTTyKeyValueRow {
            let row = MQTTyKeyValueRow::default();

            let obj = self.obj();
            obj.bind_property("display_mode", &row, "display_mode")
                .sync_create()
                .build();

            let row_model = self.row_model();

            let signal_id: Rc<RefCell<Option<glib::SignalHandlerId>>> = Default::default();

            *signal_id.borrow_mut() = Some(row.connect_closure(
                "changed",
                false,
                glib::closure_local!(
                    #[weak(rename_to = this)]
                    self,
                    #[strong]
                    signal_id,
                    #[weak]
                    row_model,
                    move |row: MQTTyKeyValueRow| {
                        row.disconnect(signal_id.take().unwrap());

                        row.set_user_changed(true);

                        row_model.append(&this.new_trigger_row());
                    }
                ),
            ));

            row.connect_closure(
                "deleted",
                false,
                glib::closure_local!(move |row: &MQTTyKeyValueRow| {
                    if let Some(idx) = row_model.find(row) {
                        row_model.remove(idx);
                    }
                }),
            );

            row
        }

        pub fn entries(&self) -> Vec<MQTTyKeyValue> {
            let row_model = self.row_model();

            let v = row_model
                .into_iter()
                .rev()
                .skip(1)
                .rev()
                .map(|i| MQTTyKeyValue::from(i.unwrap().downcast::<MQTTyKeyValueRow>().unwrap()))
                .collect::<Vec<_>>();

            // We take all but the latest
            // v.pop();

            // This does the same, I picked this one as you can see above
            // .rev().skip(1).rev()

            v
        }

        pub fn set_entries(&self, entries: &[MQTTyKeyValue]) {
            let row_model = self.row_model();

            let v = entries
                .into_iter()
                .map(|i| {
                    let row = MQTTyKeyValueRow::from(i.clone());
                    row.set_user_changed(true);
                    row
                })
                .chain(iter::once(self.new_trigger_row()))
                .collect::<Vec<_>>();

            row_model.splice(0, row_model.n_items(), &v);
        }
    }
}

glib::wrapper! {
    pub struct MQTTyPublishUserPropsTab(ObjectSubclass<imp::MQTTyPublishUserPropsTab>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
