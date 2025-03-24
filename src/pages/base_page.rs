use std::cell::{Cell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{glib, ClosureExpression};

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/pages/base_page.ui")]
    #[properties(wrapper_type = super::MQTTyBasePage)]
    pub struct MQTTyBasePage {
        #[property(get, set)]
        content: RefCell<Option<gtk::Widget>>,

        #[property(get, set)]
        sidebar: RefCell<Option<gtk::Widget>>,

        #[property(construct_only, get)]
        nav_view: RefCell<adw::NavigationView>,

        #[property(get, set)]
        is_header_bar_visible: Cell<bool>,

        #[template_child]
        sidebar_button: TemplateChild<gtk::ToggleButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyBasePage {
        const NAME: &'static str = "MQTTyBasePage";

        type Type = super::MQTTyBasePage;

        type ParentType = adw::NavigationPage;

        const ABSTRACT: bool = true;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyBasePage {
        // TODO: Needs investigation, expression gets executed twice for every page pushed onto stack,
        // expected to be executed once for every page, although ideally should be executed only once
        // for all of the lifetime of the page
        fn constructed(&self) {
            self.parent_constructed();

            let this_page_has_previous = self
                .nav_view
                .borrow()
                .property_expression_weak("visible-page")
                .chain_closure::<bool>(glib::closure_local!(
                    #[weak(rename_to = nav_view)]
                    &*self.nav_view.borrow(),
                    #[upgrade_or]
                    false,
                    move |this_page: &adw::NavigationPage, _: Option<glib::Object>| {
                        nav_view.previous_page(this_page).is_some()
                    }
                ));

            let obj = self.obj();

            let some_sidebar = obj
                .property_expression_weak("sidebar")
                .chain_closure::<bool>(glib::closure!(
                    |_: Option<glib::Object>, sidebar: Option<gtk::Widget>| { sidebar.is_some() }
                ));

            some_sidebar.bind(&*self.sidebar_button, "visible", glib::Object::NONE);

            let is_header_bar_visible = ClosureExpression::new::<bool>(
                [this_page_has_previous.upcast(), some_sidebar.upcast()],
                glib::closure!(|_: Option<glib::Object>,
                                this_page_has_previous: bool,
                                some_sidebar: bool| {
                    this_page_has_previous || some_sidebar
                }),
            );

            is_header_bar_visible.bind(&*obj, "is_header_bar_visible", Some(&*obj));
        }
    }
    impl WidgetImpl for MQTTyBasePage {}
    impl NavigationPageImpl for MQTTyBasePage {}
}

glib::wrapper! {
    pub struct MQTTyBasePage(ObjectSubclass<imp::MQTTyBasePage>)
        @extends gtk::Widget, adw::NavigationPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTyBasePage {
    pub fn new<'a>(
        content: &impl IsA<gtk::Widget>,
        sidebar: Option<&impl IsA<gtk::Widget>>,
        nav_view: &impl IsA<adw::NavigationView>,
    ) -> Self {
        glib::Object::builder()
            .property("content", content)
            .property("sidebar", sidebar)
            .property("nav_view", nav_view)
            .build()
    }
}

pub trait MQTTyBasePageImpl: NavigationPageImpl {}

unsafe impl<T: MQTTyBasePageImpl> IsSubclassable<T> for MQTTyBasePage {}
