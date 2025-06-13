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

mod publish_auth_tab;
mod publish_body_tab;
mod publish_general_tab;
mod publish_user_props_tab;
mod publish_view_notebook;

pub use publish_auth_tab::MQTTyPublishAuthTab;
pub use publish_body_tab::MQTTyPublishBodyTab;
pub use publish_general_tab::MQTTyPublishGeneralTab;
pub use publish_user_props_tab::MQTTyPublishUserPropsTab;
pub use publish_view_notebook::MQTTyPublishViewNotebook;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib;

use crate::display_mode::{MQTTyDisplayMode, MQTTyDisplayModeIface};
use crate::subclass::prelude::*;

fn handle_gesture_claim_event(ev: &gtk::GestureSingle, picked: &gtk::Widget) {
    let is_tab = glib::Type::from_name("AdwTab")
        .map(|ty| picked.type_().is_a(ty) || picked.ancestor(ty).is_some())
        .unwrap_or(false);

    let is_button =
        picked.is::<gtk::Button>() || picked.ancestor(gtk::Button::static_type()).is_some();

    if (!is_tab && !is_button)
        || (is_button
            && (ev.current_button() == 3 || ev.downcast_ref::<gtk::GestureDrag>().is_some()))
    {
        ev.set_state(gtk::EventSequenceState::Claimed);
    }
}

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/publish_view/publish_view.ui")]
    #[properties(wrapper_type = super::MQTTyPublishView)]
    pub struct MQTTyPublishView {
        #[property(get, set, override_interface = MQTTyDisplayModeIface)]
        display_mode: Cell<MQTTyDisplayMode>,

        #[template_child]
        pub tab_view: TemplateChild<adw::TabView>,

        #[template_child]
        tab_bar: TemplateChild<adw::TabBar>,

        #[template_child]
        stack: TemplateChild<gtk::Stack>,

        #[template_child]
        send_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyPublishView {
        const NAME: &'static str = "MQTTyPublishView";

        type Type = super::MQTTyPublishView;

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
    impl ObjectImpl for MQTTyPublishView {
        fn constructed(&self) {
            self.parent_constructed();

            let stack = &self.stack;
            let send_button = &self.send_button;

            self.tab_view.connect_n_pages_notify(glib::clone!(
                #[weak]
                stack,
                #[weak]
                send_button,
                move |tab_view| {
                    let n_pages = tab_view.n_pages();
                    stack.set_visible_child_name(if n_pages == 0 { "no-tabs" } else { "tabs" });

                    send_button.set_visible(n_pages != 0);
                }
            ));

            let click = gtk::GestureClick::new();
            click.set_button(0);
            click.connect_pressed(|click, n_presses, x, y| {
                if n_presses > 1 {
                    click.set_state(gtk::EventSequenceState::Claimed);
                    return;
                }
                let picked = click
                    .widget()
                    .unwrap()
                    .pick(x, y, gtk::PickFlags::DEFAULT)
                    .unwrap();
                handle_gesture_claim_event(
                    click.upcast_ref(),
                    &picked,
                );
            });

            let drag = gtk::GestureDrag::new();
            drag.connect_drag_begin(|drag, x, y| {
                let picked = drag
                    .widget()
                    .unwrap()
                    .pick(x, y, gtk::PickFlags::DEFAULT)
                    .unwrap();

                let signal_id: Rc<RefCell<Option<glib::SignalHandlerId>>> = Default::default();

                *signal_id.borrow_mut() = Some(drag.connect_drag_update(glib::clone!(
                    #[strong]
                    signal_id,
                    move |drag, _x, _y| {
                        drag.disconnect(signal_id.take().unwrap());
                        handle_gesture_claim_event(
                            drag.upcast_ref(),
                            &picked,
                        );
                    }
                )));
            });

            self.tab_bar.add_controller(click);
            self.tab_bar.add_controller(drag);
        }
    }
    impl WidgetImpl for MQTTyPublishView {}
    impl BinImpl for MQTTyPublishView {}
    impl MQTTyDisplayModeIfaceImpl for MQTTyPublishView {}
}

glib::wrapper! {
    pub struct MQTTyPublishView(ObjectSubclass<imp::MQTTyPublishView>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTyPublishView {
    pub fn tab_view(&self) -> &adw::TabView {
        &*self.imp().tab_view
    }

    pub fn new_tab(&self) {
        let notebook = MQTTyPublishViewNotebook::new();
        self.bind_property("display_mode", &notebook, "display_mode")
            .sync_create()
            .build();

        let topic_expr = notebook
            .property_expression_weak("topic")
            .chain_closure::<String>(glib::closure!(
                move |_: Option<glib::Object>, topic: String| {
                    if topic.is_empty() {
                        gettext("(untitled)")
                    } else {
                        topic
                    }
                }
            ));

        let page = self.imp().tab_view.append(&notebook);

        topic_expr.bind(&page, "title", glib::Object::NONE);

        // We create a tooltip based on topic and url values, so that users knows how to
        // differentiate between similar messages
        gtk::ClosureExpression::new::<String>(
            [
                topic_expr.upcast(),
                notebook.property_expression_weak("url").upcast(),
            ],
            glib::closure!(move |_: Option<glib::Object>, topic: String, url: String| {
                if url.is_empty() {
                    topic
                } else {
                    [topic, url].join("\r\n")
                }
            }),
        )
        .bind(&page, "tooltip", glib::Object::NONE);
    }
}
