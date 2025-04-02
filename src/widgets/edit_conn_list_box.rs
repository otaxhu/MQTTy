use std::cell::{Cell, RefCell};
use std::sync::LazyLock;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::glib::subclass::Signal;

use crate::gsettings::MQTTySettingConnection;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[properties(wrapper_type = super::MQTTyEditConnListBox)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/edit_conn_list_box.ui")]
    pub struct MQTTyEditConnListBox {
        #[property(get, set)]
        conn_model: RefCell<MQTTySettingConnection>,

        #[property(get, set, construct)]
        editing: Cell<bool>,

        #[template_child]
        url_row: TemplateChild<adw::EntryRow>,

        #[template_child]
        topic_row: TemplateChild<adw::EntryRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyEditConnListBox {
        const NAME: &'static str = "MQTTyEditConnListBox";

        type Type = super::MQTTyEditConnListBox;

        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyEditConnListBox {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            obj.connect_conn_model_notify(|obj| {
                let conn_model = obj.conn_model();

                let private = obj.imp();

                conn_model
                    .bind_property("url", &*private.url_row, "text")
                    .bidirectional()
                    .sync_create()
                    .build();

                conn_model
                    .bind_property("topic", &*private.topic_row, "text")
                    .bidirectional()
                    .sync_create()
                    .build();
            });
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
                vec![
                    Signal::builder("saving-conn").build(),
                    Signal::builder("deleting-conn").build(),
                ]
            });
            &*SIGNALS
        }
    }
    impl WidgetImpl for MQTTyEditConnListBox {}
    impl BinImpl for MQTTyEditConnListBox {}

    #[gtk::template_callbacks]
    impl MQTTyEditConnListBox {
        #[template_callback]
        fn on_save_conn(&self) {
            let obj = self.obj();

            obj.emit_by_name::<()>("saving-conn", &[]);
        }

        #[template_callback]
        fn on_delete_conn(&self) {
            let obj = self.obj();

            obj.emit_by_name::<()>("deleting-conn", &[]);
        }
    }
}

glib::wrapper! {
    pub struct MQTTyEditConnListBox(ObjectSubclass<imp::MQTTyEditConnListBox>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
