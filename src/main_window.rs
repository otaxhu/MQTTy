use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::application::MQTTyApplication;
use crate::config;
// use crate::gsettings::OpenConnection;

mod imp {

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/main_window.ui")]
    pub struct MQTTyWindow {
        #[template_child]
        split_view: TemplateChild<adw::OverlaySplitView>,

        #[template_child]
        stack: TemplateChild<gtk::Stack>,

        #[template_child]
        conn_stack: TemplateChild<gtk::Stack>,

        #[template_child]
        sidebar_button: TemplateChild<gtk::ToggleButton>,

        // #[template_child]
        // grid_view: TemplateChild<gtk::GridView>,
        // #[template_child]
        // conn_list_store: TemplateChild<gtk::StringList>,
        #[template_child]
        flowbox: TemplateChild<gtk::FlowBox>,
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

            // Load latest window state
            obj.load_window_size();

            let split_view = &self.split_view;

            // Close sidebar when selecting page, only when using mobile
            self.stack.connect_visible_child_notify(glib::clone!(
                #[weak]
                split_view,
                move |_| {
                    split_view.set_show_sidebar(!split_view.is_collapsed());
                }
            ));

            let sidebar_button = &self.sidebar_button;

            // Only show sidebar_button when the visible StackPage is conn_stack_conn_panel
            self.conn_stack.connect_visible_child_notify(glib::clone!(
                #[weak]
                sidebar_button,
                move |conn_stack| {
                    sidebar_button.set_visible(
                        conn_stack.visible_child_name().unwrap() == "conn_stack_conn_panel",
                    );
                }
            ));

            // let app = MQTTyApplication::get_singleton();

            // let settings = app.settings();

            // let s = settings.get::<Vec<OpenConnection>>("open-connections");

            // let store = gio::ListStore::new::<gtk::Widget>();

            // TODO: Append the first element, the "create" widget, it will be a card, when clicked,
            // it will prompt the user for connection configuration, when saved, that configuration will
            // be displayed as another card appended to store.
            //
            // store.append(...);

            // self.flowbox.bind_model(Some(&store), |item| {
            //     let string = item.downcast_ref::<gtk::StringObject>().unwrap();
            //     Label::builder().label(string).build().into()
            // });
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
}
