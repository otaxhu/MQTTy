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
use formatx::formatx;
use gettextrs::gettext;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::application::MQTTyApplication;
use crate::config;
use crate::toast::MQTTyToastBuilder;
use crate::widgets::{MQTTyPublishView, MQTTyPublishViewNotebook, MQTTySubscriptionsView};

mod imp {

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/main_window.ui")]
    pub struct MQTTyWindow {
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,

        #[template_child]
        view_stack: TemplateChild<adw::ViewStack>,

        #[template_child]
        publish_view: TemplateChild<MQTTyPublishView>,

        #[template_child]
        subscriptions_view: TemplateChild<MQTTySubscriptionsView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyWindow {
        const NAME: &'static str = "MQTTyWindow";
        type Type = super::MQTTyWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MQTTyWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Devel Profile
            if config::PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            if !adw::StyleManager::default().is_system_supports_accent_colors() {
                obj.add_css_class("accent-color-system-unsupported");
            }

            // Load latest window state
            obj.load_window_size();

            let publish_view = &self.publish_view;
            let publish_tab_view = publish_view.tab_view();

            let action_publish_send = gio::SimpleAction::new("publish-send", None);
            action_publish_send.connect_activate(glib::clone!(
                #[weak]
                publish_tab_view,
                #[weak]
                obj,
                move |_, _| {
                    let publishing_toast = MQTTyToastBuilder::new()
                        .title(gettext("Publishing message..."))
                        .timeout(3)
                        .build();

                    obj.toast(&publishing_toast);

                    let notebook = publish_tab_view
                        .selected_page()
                        .unwrap()
                        .child()
                        .downcast::<MQTTyPublishViewNotebook>()
                        .unwrap();

                    glib::spawn_future_local(async move {
                        let ret = notebook.send().await;

                        publishing_toast.dismiss();

                        let toast = match ret {
                            Ok(_) => MQTTyToastBuilder::new()
                                .title(
                                    formatx!(
                                        gettext("Message published to topic {}"),
                                        notebook.topic()
                                    )
                                    .unwrap(),
                                )
                                .icon(
                                    gtk::Image::builder()
                                        .icon_name("object-select-symbolic")
                                        .css_classes(["success"])
                                        .build()
                                        .as_ref(),
                                )
                                .timeout(3)
                                .build(),
                            Err(e) => MQTTyToastBuilder::new()
                                .title(formatx!(gettext("Error while publishing: {}"), e).unwrap())
                                .icon(
                                    gtk::Image::builder()
                                        .icon_name("network-error-symbolic")
                                        .build()
                                        .as_ref(),
                                )
                                .timeout(3)
                                .build(),
                        };

                        obj.toast(&toast);
                    });
                }
            ));

            let action_publish_new_tab = gio::SimpleAction::new("publish-new-tab", None);
            action_publish_new_tab.connect_activate(glib::clone!(
                #[weak]
                publish_view,
                move |_, _| {
                    publish_view.new_tab();
                }
            ));

            let action_publish_delete_tab = gio::SimpleAction::new("publish-delete-tab", None);
            action_publish_delete_tab.connect_activate(glib::clone!(
                #[weak]
                publish_tab_view,
                move |_, _| {
                    publish_tab_view.close_page(&publish_tab_view.selected_page().unwrap());
                }
            ));

            let view_stack = &self.view_stack;

            let in_publish_view = view_stack
                .property_expression_weak("visible-child-name")
                .chain_closure::<bool>(glib::closure!(
                    move |_: Option<glib::Object>, name: &str| name == "publish"
                ));

            let n_tabs = publish_tab_view.property_expression_weak("n-pages");

            let has_pages_and_in_publish_view = gtk::ClosureExpression::new::<bool>(
                [in_publish_view.upcast_ref(), &n_tabs],
                glib::closure!(move |_: Option<glib::Object>,
                                     in_publish_view: bool,
                                     n_tabs: i32| {
                    in_publish_view && n_tabs != 0
                }),
            );
            has_pages_and_in_publish_view.bind(&action_publish_send, "enabled", glib::Object::NONE);
            has_pages_and_in_publish_view.bind(
                &action_publish_delete_tab,
                "enabled",
                glib::Object::NONE,
            );
            in_publish_view.bind(&action_publish_new_tab, "enabled", glib::Object::NONE);

            obj.add_action(&action_publish_send);
            obj.add_action(&action_publish_new_tab);
            obj.add_action(&action_publish_delete_tab);

            let action_set_publish_view = gio::SimpleAction::new("set-publish-view", None);
            action_set_publish_view.connect_activate(glib::clone!(
                #[weak]
                view_stack,
                move |_, _| {
                    view_stack.set_visible_child_name("publish");
                }
            ));

            let action_set_subscriptions_view =
                gio::SimpleAction::new("set-subscriptions-view", None);
            action_set_subscriptions_view.connect_activate(glib::clone!(
                #[weak]
                view_stack,
                move |_, _| {
                    view_stack.set_visible_child_name("subscriptions");
                }
            ));

            obj.add_action(&action_set_publish_view);
            obj.add_action(&action_set_subscriptions_view);

            let subscriptions_view = &self.subscriptions_view;

            let action_subscriptions_new =
                gio::SimpleAction::new("subscriptions-new-connection", None);
            action_subscriptions_new.connect_activate(glib::clone!(
                #[weak]
                subscriptions_view,
                move |_, _| {
                    glib::spawn_future_local(async move {
                        subscriptions_view.new_connection().await;
                    });
                }
            ));
            let in_subscriptions_view = in_publish_view.chain_closure::<bool>(glib::closure!(
                |_: Option<glib::Object>, in_publish_view: bool| !in_publish_view
            ));

            in_subscriptions_view.bind(&action_subscriptions_new, "enabled", glib::Object::NONE);

            obj.add_action(&action_subscriptions_new);
        }
    }

    impl WidgetImpl for MQTTyWindow {}
    impl WindowImpl for MQTTyWindow {
        // Save window state on delete event
        fn close_request(&self) -> glib::Propagation {
            self.obj().save_window_size();

            // Pass close request on to the parent
            self.parent_close_request()
        }
    }

    impl ApplicationWindowImpl for MQTTyWindow {}
    impl AdwApplicationWindowImpl for MQTTyWindow {}
}

glib::wrapper! {
    pub struct MQTTyWindow(ObjectSubclass<imp::MQTTyWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Root;
}

impl MQTTyWindow {
    pub fn new(app: &MQTTyApplication) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    fn save_window_size(&self) {
        let app = MQTTyApplication::get_singleton();

        let (width, height) = self.default_size();

        let settings = app.settings();

        settings.set_int("window-width", width).unwrap();
        settings.set_int("window-height", height).unwrap();

        settings
            .set_boolean("is-maximized", self.is_maximized())
            .unwrap();
    }

    fn load_window_size(&self) {
        let app = MQTTyApplication::get_singleton();

        let settings = app.settings();

        let width = settings.int("window-width");
        let height = settings.int("window-height");
        let is_maximized = settings.boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }

    pub fn toast(&self, toast: &adw::Toast) {
        self.imp().toast_overlay.add_toast(toast.clone());
    }
}
