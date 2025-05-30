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

use std::cell::OnceCell;
use std::sync::LazyLock;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::Signal;
use gtk::{gio, glib};

use crate::client::MQTTyClientSubscriptionsData;
use crate::widgets::subscriptions_view::MQTTySubscriptionsConnectionRow;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(
        resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/subscriptions_connections.ui"
    )]
    #[properties(wrapper_type = super::MQTTySubscriptionsConnections)]
    pub struct MQTTySubscriptionsConnections {
        #[property(get, set)]
        model: OnceCell<gio::ListStore>,

        #[template_child]
        list_box: TemplateChild<gtk::ListBox>,

        #[template_child]
        stack: TemplateChild<gtk::Stack>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionsConnections {
        const NAME: &'static str = "MQTTySubscriptionsConnections";

        type Type = super::MQTTySubscriptionsConnections;

        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTySubscriptionsConnections {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let list_box = &self.list_box;

            let stack = &self.stack;

            obj.connect_model_notify(glib::clone!(
                #[weak]
                list_box,
                #[weak]
                stack,
                move |obj| {
                    let model = obj.model();

                    model.connect_notify_local(Some("n-items"), move |list, _| {
                        stack.set_visible_child_name(if list.n_items() != 0 {
                            "subscriptions"
                        } else {
                            "no-subscriptions"
                        });
                    });

                    list_box.bind_model(Some(&model), |o| {
                        let data = o.downcast_ref::<MQTTyClientSubscriptionsData>().unwrap();

                        MQTTySubscriptionsConnectionRow::from(data.clone()).upcast()
                    });
                }
            ));
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
                vec![Signal::builder("row-activated")
                    .param_types([MQTTySubscriptionsConnectionRow::static_type()])
                    .build()]
            });
            &*SIGNALS
        }
    }

    impl WidgetImpl for MQTTySubscriptionsConnections {}
    impl NavigationPageImpl for MQTTySubscriptionsConnections {}

    #[gtk::template_callbacks]
    impl MQTTySubscriptionsConnections {
        #[template_callback]
        fn on_row_activated(&self, row: &gtk::ListBoxRow) {
            let obj = self.obj();
            obj.emit_by_name::<()>(
                "row-activated",
                &[row
                    .downcast_ref::<MQTTySubscriptionsConnectionRow>()
                    .unwrap()],
            );
        }
    }
}

glib::wrapper! {
    pub struct MQTTySubscriptionsConnections(ObjectSubclass<imp::MQTTySubscriptionsConnections>)
        @extends gtk::Widget, adw::NavigationPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTySubscriptionsConnections {
    pub fn connect_row_activated(
        &self,
        f: impl Fn(&Self, MQTTyClientSubscriptionsData) + 'static,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "connection-selected",
            false,
            glib::closure_local!(|o: &Self, data: MQTTyClientSubscriptionsData| f(o, data)),
        )
    }
}
