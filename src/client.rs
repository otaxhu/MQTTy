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

mod message;

pub use message::MQTTyClientMessage;

use std::cell::{Cell, OnceCell, RefCell};
use std::sync::LazyLock;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::glib::subclass::Signal;

#[derive(Default, Clone, Copy, glib::Enum, PartialEq)]
#[enum_type(name = "MQTTyClientVersion")]
pub enum MQTTyClientVersion {
    #[default]
    V3X,
    V5,
}

#[derive(Default, Clone, Copy, glib::Enum)]
#[enum_type(name = "MQTTyClientQos")]
pub enum MQTTyClientQos {
    #[default]
    Qos0,
    Qos1,
    Qos2,
}

mod imp {

    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::MQTTyClient)]
    pub struct MQTTyClient {
        #[property(get, construct_only)]
        url: RefCell<String>,

        #[property(get, construct_only, builder(MQTTyClientVersion::V3X))]
        mqtt_version: Cell<MQTTyClientVersion>,

        #[property(get, construct_only)]
        username: RefCell<String>,

        #[property(get, construct_only)]
        password: RefCell<String>,

        client: OnceCell<paho::AsyncClient>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyClient {
        const NAME: &'static str = "MQTTyClient";

        type Type = super::MQTTyClient;

        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTyClient {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            let client = match paho::CreateOptionsBuilder::new()
                .server_uri(obj.url())
                .create_client()
            {
                Err(e) => panic!("CLIENT CREATION ERROR {:?}", e),
                Ok(c) => c,
            };

            // Receiving message signal and redirecting it to Object signal emission
            let (message_tx, message_rx) = async_channel::bounded(1);

            client.set_message_callback(move |_, msg| {
                let msg = msg.unwrap();
                let _ = message_tx.send_blocking(msg);
            });

            glib::spawn_future_local(glib::clone!(
                #[weak]
                obj,
                async move {
                    loop {
                        let Ok(msg) = message_rx.recv().await else {
                            return;
                        };

                        println!("{:?}", msg);

                        let out_msg = MQTTyClientMessage::new();

                        let props = msg.properties();

                        out_msg.set_topic(msg.topic());
                        out_msg.set_qos(MQTTyClientQos::from(msg.qos()));
                        out_msg.set_body(msg.payload());
                        out_msg.set_mqtt_version(obj.mqtt_version());
                        out_msg.set_content_type(props.get_string(paho::PropertyCode::ContentType));
                        out_msg.set_retained(msg.retained());
                        out_msg
                            .set_user_properties(props.user_iter().collect::<Vec<_>>().as_slice());

                        obj.emit_by_name::<()>("message", &[&out_msg]);
                    }
                }
            ));

            self.client.set(client).ok().unwrap();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
                vec![Signal::builder("message")
                    .param_types([MQTTyClientMessage::static_type()])
                    .build()]
            });
            &*SIGNALS
        }
    }

    impl MQTTyClient {
        fn client(&self) -> &paho::AsyncClient {
            self.client.get().unwrap()
        }

        pub async fn connect_client(&self) -> Result<(), String> {
            let client = self.client();

            let obj = self.obj();

            client
                .connect(Some(
                    paho::ConnectOptionsBuilder::with_mqtt_version(obj.mqtt_version())
                        .user_name(obj.username())
                        .password(obj.password())
                        .ssl_options(Default::default())
                        .finalize(),
                ))
                .await
                .map(|res| println!("CONNECTION SERVER RESPONSE: {res:?}"))
                .map_err(|e| e.to_string())
        }

        pub async fn disconnect_client(&self) -> Result<(), String> {
            let client = self.client();

            client
                .disconnect(None)
                .await
                .map(|res| println!("DISCONNECTION SERVER RESPONSE: {res:?}"))
                .map_err(|e| e.to_string())
        }

        pub async fn publish(&self, message: &MQTTyClientMessage) -> Result<(), String> {
            let client = self.client();

            client
                .publish(paho::Message::from(message))
                .await
                .map_err(|e| e.to_string())
        }

        pub async fn subscribe(&self, topic: &str, qos: MQTTyClientQos) -> Result<(), String> {
            let client = self.client();

            client
                .subscribe(topic, qos)
                .await
                .map(|res| println!("SUBSCRIPTION SERVER RESPONSE: {res:?}"))
                .map_err(|e| e.to_string())
        }
    }
}

glib::wrapper! {
    /// This Object works as an inteface, in case the underlying MQTT library changes,
    /// also, we are using it so that we can emit signals like "connected",
    /// "connection-error" and "message"
    pub struct MQTTyClient(ObjectSubclass<imp::MQTTyClient>);
}

impl MQTTyClient {
    pub fn new(
        url: &str,
        mqtt_version: MQTTyClientVersion,
        username: &str,
        password: &str,
    ) -> Self {
        glib::Object::builder()
            .property("url", url)
            .property("mqtt_version", mqtt_version)
            .property("username", username)
            .property("password", password)
            .build()
    }

    pub async fn connect_client(&self) -> Result<(), String> {
        self.imp().connect_client().await
    }

    pub async fn disconnect_client(&self) -> Result<(), String> {
        self.imp().disconnect_client().await
    }

    pub async fn publish(&self, message: &MQTTyClientMessage) -> Result<(), String> {
        self.imp().publish(message).await
    }

    pub async fn subscribe(&self, topic: &str, qos: MQTTyClientQos) -> Result<(), String> {
        self.imp().subscribe(topic, qos).await
    }

    pub fn connect_message(
        &self,
        cb: impl Fn(&Self, &MQTTyClientMessage) + 'static,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "message",
            false,
            glib::closure_local!(move |o: &Self, msg: &MQTTyClientMessage| cb(o, msg)),
        )
    }
}

/*
    ======== PAHO ADAPTOR CODE ========
*/

impl From<&MQTTyClientMessage> for paho::Message {
    fn from(value: &MQTTyClientMessage) -> Self {
        let mut props = paho::Properties::new();

        if let Some(content_type) = value.content_type() {
            props
                .push_string(paho::PropertyCode::ContentType, &content_type)
                .unwrap();
        }

        for (key, value) in value.user_properties().into_iter() {
            props
                .push_string_pair(paho::PropertyCode::UserProperty, &key, &value)
                .unwrap();
        }

        let msg = paho::MessageBuilder::new()
            .topic(value.topic())
            .qos(paho::QoS::from(value.qos()))
            .retained(value.retained())
            .payload(value.body())
            .properties(props)
            .finalize();

        msg
    }
}

// // NOTE:
// //
// // Cannot implement this trait succesfully for paho crate, since there is no way
// // to know the MQTT version that belongs to the paho::Message struct,
// // we could assume that a message with empty properties is v3.x and others
// // with properties is v5, but v5 messages can also come with empty props.
// //
// // As a quick fix, we are manually setting the mqtt_version prop for MQTTyClientMessage,
// // depending on mqtt_version prop in the client.
//
// impl From<&paho::Message> for MQTTyClientMessage {
//     fn from(value: &paho::Message) -> Self {
//         let props = value.properties();
//
//         let message = MQTTyClientMessage::new();
//
//         message.set_topic(value.topic());
//         message.set_qos(MQTTyClientQos::from(value.qos()));
//         message.set_retained(value.retained());
//         message.set_body(value.payload());
//         message.set_content_type(
//             props
//                 .get_string(paho::PropertyCode::ContentType)
//                 .unwrap_or("".to_string()),
//         );
//         message.set_mqtt_version(...);
//
//         message
//     }
// }

impl From<MQTTyClientVersion> for paho::MqttVersion {
    fn from(value: MQTTyClientVersion) -> Self {
        match value {
            MQTTyClientVersion::V3X => paho::MqttVersion::Default,
            MQTTyClientVersion::V5 => paho::MqttVersion::V5,
        }
    }
}

impl From<paho::MqttVersion> for MQTTyClientVersion {
    fn from(value: paho::MqttVersion) -> Self {
        match value {
            paho::MqttVersion::V5 => MQTTyClientVersion::V5,
            _ => MQTTyClientVersion::V3X,
        }
    }
}

impl From<MQTTyClientQos> for paho::QoS {
    fn from(value: MQTTyClientQos) -> Self {
        match value {
            MQTTyClientQos::Qos0 => paho::QoS::AtMostOnce,
            MQTTyClientQos::Qos1 => paho::QoS::AtLeastOnce,
            MQTTyClientQos::Qos2 => paho::QoS::ExactlyOnce,
        }
    }
}

impl From<paho::QoS> for MQTTyClientQos {
    fn from(value: paho::QoS) -> Self {
        match value {
            paho::QoS::AtMostOnce => MQTTyClientQos::Qos0,
            paho::QoS::AtLeastOnce => MQTTyClientQos::Qos1,
            paho::QoS::ExactlyOnce => MQTTyClientQos::Qos2,
        }
    }
}
