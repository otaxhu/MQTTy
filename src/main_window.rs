use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::application::MQTTyApplication;
use crate::config;
use crate::pages::MQTTyAllConnPage;
use crate::pages::MQTTyBasePage;
use crate::widgets::MQTTyAddConnCard;
use crate::widgets::MQTTyBaseCard;

mod imp {

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/main_window.ui")]
    pub struct MQTTyWindow {
        #[template_child]
        nav_view: TemplateChild<adw::NavigationView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyWindow {
        const NAME: &'static str = "MQTTyWindow";
        type Type = super::MQTTyWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            MQTTyBasePage::static_type();
            MQTTyAllConnPage::static_type();
            MQTTyBaseCard::static_type();
            MQTTyAddConnCard::static_type();

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
