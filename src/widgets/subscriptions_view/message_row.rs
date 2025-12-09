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

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib;

use crate::client::{MQTTyClientMessage, MQTTyClientQos};
use crate::content_type::MQTTyContentType;
use crate::l10n;
use crate::widgets::MQTTySourceView;

mod imp {

    use super::*;

    #[derive(Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/subscriptions_view/message_row.ui")]
    #[properties(wrapper_type = super::MQTTySubscriptionsMessageRow)]
    pub struct MQTTySubscriptionsMessageRow {
        #[property(get, construct_only)]
        message: OnceCell<MQTTyClientMessage>,

        #[template_child]
        timestamp_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        retained_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        content_type_dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        source_view: TemplateChild<MQTTySourceView>,
        #[template_child]
        body_row_stack: TemplateChild<gtk::Stack>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionsMessageRow {
        const NAME: &'static str = "MQTTySubscriptionsMessageRow";

        type Type = super::MQTTySubscriptionsMessageRow;

        type ParentType = adw::ExpanderRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::types::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTySubscriptionsMessageRow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let msg = obj.message();

            msg.bind_property("topic", &*obj, "title")
                .sync_create()
                .build();

            let qos_expr =
                msg.property_expression_weak("qos")
                    .chain_closure::<String>(glib::closure!(
                        |_: Option<glib::Object>, qos: MQTTyClientQos| qos.translated()
                    ));

            qos_expr.bind(&*obj, "subtitle", glib::Object::NONE);

            let timestamp_expr = msg
                .property_expression_weak("timestamp")
                .chain_closure::<String>(glib::closure!(
                    |_: Option<glib::Object>, timestamp: &str| l10n::format_datetime(timestamp)
                        .unwrap()
                ));

            timestamp_expr.bind(&*self.timestamp_row, "subtitle", glib::Object::NONE);

            let retained_expr = msg
                .property_expression_weak("retained")
                .chain_closure::<String>(glib::closure!(
                    |_: Option<glib::Object>, retained: bool| if retained {
                        gettext("Yes")
                    } else {
                        gettext("No")
                    }
                ));

            retained_expr.bind(&*self.retained_row, "subtitle", glib::Object::NONE);

            let list = gtk::StringList::new(&[]);

            // Don't use None content type
            for i in MQTTyContentType::listed()[1..].iter() {
                list.append(&i.translated());
            }

            self.content_type_dropdown.set_model(Some(&list));
            let content_type_expr = self
                .content_type_dropdown
                .property_expression_weak("selected")
                .chain_closure::<MQTTyContentType>(glib::closure!(
                    |_: Option<glib::Object>, i: u32| MQTTyContentType::listed()[(i + 1) as usize]
                ));

            let selected_language_expr = content_type_expr
                .chain_closure::<Option<sourceview::Language>>(glib::closure!(
                    |_: Option<glib::Object>, content_type: MQTTyContentType| {
                        let language_manager = sourceview::LanguageManager::default();

                        match content_type {
                            MQTTyContentType::None | MQTTyContentType::Raw => None,
                            MQTTyContentType::Json => language_manager.language("json"),
                            MQTTyContentType::Xml => language_manager.language("xml"),
                        }
                    }
                ));

            let buffer = self.source_view.buffer();

            selected_language_expr.bind(&buffer, "language", glib::Object::NONE);

            let body = msg.body();

            // TODO: Implement a HEX editor that fully shows msg.body() in its
            // HEX form.
            buffer.set_text(&String::from_utf8_lossy(&body));

            self.body_row_stack
                .set_visible_child_name(if body.len() == 0 { "no-body" } else { "body" });
        }
    }
    impl WidgetImpl for MQTTySubscriptionsMessageRow {}
    impl ListBoxRowImpl for MQTTySubscriptionsMessageRow {}
    impl PreferencesRowImpl for MQTTySubscriptionsMessageRow {}
    impl ExpanderRowImpl for MQTTySubscriptionsMessageRow {}
}

glib::wrapper! {
    pub struct MQTTySubscriptionsMessageRow(ObjectSubclass<imp::MQTTySubscriptionsMessageRow>)
         @extends adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget, adw::ExpanderRow,
         @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Actionable;
}

impl MQTTySubscriptionsMessageRow {
    pub fn new(msg: &MQTTyClientMessage) -> Self {
        glib::Object::builder().property("message", msg).build()
    }
}
