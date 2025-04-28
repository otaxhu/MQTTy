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

use std::cell::Cell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib;

use crate::display_mode::{MQTTyDisplayMode, MQTTyDisplayModeIface};
use crate::subclass::prelude::*;

mod imp {

    use super::*;

    #[derive(gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/publish_view/publish_view.ui")]
    #[properties(wrapper_type = super::MQTTyPublishView)]
    pub struct MQTTyPublishView {
        #[property(get, set, override_interface = MQTTyDisplayModeIface)]
        display_mode: Cell<MQTTyDisplayMode>,

        #[template_child]
        tab_view: TemplateChild<adw::TabView>,

        #[template_child]
        stack: TemplateChild<gtk::Stack>,

        #[template_child]
        send_button: TemplateChild<gtk::Button>,
    }

    impl Default for MQTTyPublishView {
        fn default() -> Self {
            Self {
                display_mode: Cell::new(MQTTyDisplayMode::Desktop),
                tab_view: Default::default(),
                stack: Default::default(),
                send_button: Default::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyPublishView {
        const NAME: &'static str = "MQTTyPublishView";

        type Type = super::MQTTyPublishView;

        type ParentType = adw::Bin;

        type Interfaces = (MQTTyDisplayModeIface,);

        fn class_init(klass: &mut Self::Class) {
            klass.install_action("publish-view.new-tab", None, |this, _, _| {
                let notebook = MQTTyPublishViewNotebook::new();
                this.bind_property("display_mode", &notebook, "display_mode")
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

                let page = this.imp().tab_view.append(&notebook);

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
            });

            klass.install_action("publish-view.send", None, |this, _, _| {
                let notebook = this
                    .imp()
                    .tab_view
                    .selected_page()
                    .unwrap()
                    .child()
                    .downcast::<MQTTyPublishViewNotebook>()
                    .unwrap();

                glib::spawn_future_local(async move {
                    notebook.send().await.unwrap();
                });
            });

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
