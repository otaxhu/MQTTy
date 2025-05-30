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

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::client::{MQTTyClient, MQTTyClientMessage, MQTTyClientQos, MQTTyClientVersion};
use crate::content_type::MQTTyContentType;
use crate::display_mode::{MQTTyDisplayMode, MQTTyDisplayModeIface};
use crate::subclass::prelude::*;
use crate::utils;
use crate::widgets::MQTTyPublishUserPropsTab;

mod imp {

    use super::*;

    #[derive(gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/otaxhu/MQTTy/ui/publish_view/publish_view_notebook.ui")]
    #[properties(wrapper_type = super::MQTTyPublishViewNotebook)]
    pub struct MQTTyPublishViewNotebook {
        pub client: OnceCell<MQTTyClient>,

        #[property(get, set, override_interface = MQTTyDisplayModeIface)]
        display_mode: Cell<MQTTyDisplayMode>,

        #[property(get, set, builder(Default::default()))]
        mqtt_version: Cell<MQTTyClientVersion>,

        #[property(get, set)]
        topic: RefCell<String>,

        #[property(get, set)]
        url: RefCell<String>,

        #[property(get, set, builder(Default::default()))]
        qos: Cell<MQTTyClientQos>,

        #[property(get, set)]
        body: RefCell<String>,

        #[property(get, set, builder(Default::default()))]
        content_type: Cell<MQTTyContentType>,

        #[property(get, set)]
        username: RefCell<String>,

        #[property(get, set)]
        password: RefCell<String>,

        #[template_child]
        pub user_properties_tab: TemplateChild<MQTTyPublishUserPropsTab>,

        #[template_child]
        user_properties_stack: TemplateChild<gtk::Stack>,
    }

    impl Default for MQTTyPublishViewNotebook {
        fn default() -> Self {
            Self {
                display_mode: Cell::new(MQTTyDisplayMode::Desktop),
                mqtt_version: Default::default(),
                topic: Default::default(),
                url: Default::default(),
                qos: Default::default(),
                client: Default::default(),
                body: Default::default(),
                content_type: Default::default(),
                user_properties_tab: Default::default(),
                username: Default::default(),
                password: Default::default(),
                user_properties_stack: Default::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyPublishViewNotebook {
        const NAME: &'static str = "MQTTyPublishViewNotebook";

        type Type = super::MQTTyPublishViewNotebook;

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
    impl ObjectImpl for MQTTyPublishViewNotebook {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let group = gio::SimpleActionGroup::new();

            let mqtt_version_state = utils::connect_mqtt_version_action(&*obj, &group);
            utils::connect_qos_action(&*obj, &group);

            obj.insert_action_group("publish-view-notebook", Some(&group));

            mqtt_version_state
                .bind_property("state", &*self.user_properties_stack, "visible-child-name")
                .transform_to(|_, state: glib::Variant| state.str().map(String::from))
                .sync_create()
                .build();
        }
    }
    impl WidgetImpl for MQTTyPublishViewNotebook {}
    impl BinImpl for MQTTyPublishViewNotebook {}
    impl MQTTyDisplayModeIfaceImpl for MQTTyPublishViewNotebook {}
}

glib::wrapper! {
    pub struct MQTTyPublishViewNotebook(ObjectSubclass<imp::MQTTyPublishViewNotebook>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTyPublishViewNotebook {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub async fn send(&self) -> Result<(), String> {
        let mqtt_version = self.mqtt_version();

        let client = MQTTyClient::builder()
            .url(&self.url())
            .mqtt_version(mqtt_version)
            .username(&self.username())
            .password(&self.password())
            .clean_start(true) // Publish view won't have any session associated
            .build();

        client.connect_client().await?;

        let msg = MQTTyClientMessage::new();

        msg.set_topic(self.topic());
        msg.set_qos(self.qos());
        if self.content_type() != MQTTyContentType::None {
            msg.set_body(self.body().as_ref());
        }
        msg.set_mqtt_version(mqtt_version);

        // Specific to MQTT v5
        if mqtt_version == MQTTyClientVersion::V5 {
            msg.set_content_type(self.content_type().mime_type());
            msg.set_user_properties(
                self.imp()
                    .user_properties_tab
                    .entries()
                    .iter()
                    .map(|i| (i.key(), i.value()))
                    .collect::<Vec<_>>()
                    .as_ref(),
            );
        }

        client.publish(&msg).await
    }
}
