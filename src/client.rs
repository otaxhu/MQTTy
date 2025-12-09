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
use std::time;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use gtk::glib::subclass::Signal;
use serde::Serialize;

#[derive(Default, Clone, Copy, glib::Enum, PartialEq, Serialize)]
#[enum_type(name = "MQTTyClientVersion")]
pub enum MQTTyClientVersion {
    #[default]
    V3X,
    V5,
}

#[derive(Default, Clone, Copy, glib::Enum, PartialEq, Serialize)]
#[enum_type(name = "MQTTyClientQos")]
pub enum MQTTyClientQos {
    #[default]
    Qos0,
    Qos1,
    Qos2,
}

impl MQTTyClientQos {
    pub fn translated(&self) -> String {
        match self {
            MQTTyClientQos::Qos0 => gettext("QoS 0"),
            MQTTyClientQos::Qos1 => gettext("QoS 1"),
            MQTTyClientQos::Qos2 => gettext("QoS 2"),
        }
    }
}

#[derive(Clone, Copy, glib::Enum, PartialEq, Debug)]
#[enum_type(name = "MQTTyClientConnectionState")]
pub enum MQTTyClientConnectionState {
    /// Emitted when the client connects with the broker.
    Connected,
    /// Emitted when the client is disconnected gracefully, this can be either client side
    /// or server side (MQTT V5 server disconnect packet).
    Disconnected,
    /// Emitted when the connection is lost, automatic reconnection is inmediately
    /// performed.
    Reconnecting,
    /// Emitted when the client has reconnected too many times without success.
    ///
    /// Note that this will be emitted after `Self::Reconnecting` is emitted, so
    /// the order would be `Self::Reconnecting -> after too many tries -> Self::ReconnectFailure`
    ///
    /// Client won't reconnect after this is emitted. You can still reconnect by calling
    /// `connect_client()` method.
    ReconnectFailure,
    /// Emitted when the session is taken over due to duplicated client_id.
    ///
    /// Client won't reconnect after this is emitted. You can still reconnect by calling
    /// `connect_client()` method, though you will need to change the current client_id
    /// to avoid this again.
    SessionTakenOver,
}

mod imp {

    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::MQTTyClient)]
    pub struct MQTTyClient {
        #[property(get, construct_only, default = "")]
        client_id: RefCell<String>,

        #[property(get, construct_only)]
        url: RefCell<String>,

        #[property(get, construct_only, builder(MQTTyClientVersion::V3X))]
        mqtt_version: Cell<MQTTyClientVersion>,

        #[property(get, construct_only, nullable)]
        username: RefCell<Option<String>>,

        #[property(get, construct_only, nullable)]
        password: RefCell<Option<String>>,

        #[property(get, construct_only)]
        clean_start: Cell<bool>,

        #[property(get)]
        connected: Cell<bool>,

        in_reconnect: Cell<bool>,

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
                .client_id(obj.client_id())
                .create_client()
            {
                Err(e) => panic!("CLIENT CREATION ERROR {:?}", e),
                Ok(c) => c,
            };

            // Setting connected prop according to connection state
            obj.connect_connection_state_changed(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |_, state| {
                    println!("CONNECTION STATE CHANGED: {state:?}");

                    let connected = match state {
                        MQTTyClientConnectionState::Connected => true,
                        MQTTyClientConnectionState::Disconnected
                        | MQTTyClientConnectionState::Reconnecting => false,
                        _ => return,
                    };

                    this.set_connected(connected);
                }
            ));

            // We cannot use glib::clone!(...) on self because there would
            // always be a reference inside of the spawned GLib futures,
            // otherwise the client will never drop.
            let weak_this = self.downgrade();

            // Receiving the connection lost signal and handling automatic reconnect,
            // as well as applying the heuristics for detecting session takeovers and other
            // network-related problems.
            let (connection_lost_tx, connection_lost_rx) = async_channel::bounded(1);

            client.set_connection_lost_callback(move |_| {
                let _ = connection_lost_tx.send_blocking(());
            });

            glib::spawn_future_local(glib::clone!(
                #[strong]
                weak_this,
                async move {
                    // Variables for detecting session takeover
                    let window_secs = 60;
                    let max_threshold = 10;
                    let mut timestamps: Vec<time::Instant> = vec![];

                    loop {
                        let Ok(_) = connection_lost_rx.recv().await else {
                            return;
                        };

                        // Upgrading after receiving async channel.
                        let Some(this) = weak_this.upgrade() else {
                            return;
                        };
                        let obj = this.obj();

                        this.in_reconnect.set(true);
                        obj.emit_by_name::<()>(
                            "connection-state-changed",
                            &[&MQTTyClientConnectionState::Reconnecting],
                        );

                        let now = time::Instant::now();
                        timestamps.push(now);

                        let cutpoint = now - time::Duration::from_secs(window_secs);
                        timestamps.retain(|ts| *ts > cutpoint);

                        if timestamps.len() >= max_threshold {
                            // Multiple connection losses in a short amount of time.
                            //
                            // v3.x Heuristic: Assume session take over.
                            //
                            // v5 Heuristic: Assume network problems.

                            timestamps.clear();
                            this.in_reconnect.set(false);

                            let state = match obj.mqtt_version() {
                                MQTTyClientVersion::V3X => {
                                    MQTTyClientConnectionState::SessionTakenOver
                                }
                                MQTTyClientVersion::V5 => {
                                    MQTTyClientConnectionState::ReconnectFailure
                                }
                            };
                            obj.emit_by_name::<()>("connection-state-changed", &[&state]);

                            continue;
                        }

                        // We put our automatic reconnect implementation here,
                        // we don't use Paho's because we need to detect network
                        // failure (which is not exposed by Paho).

                        let res = backoff::future::retry(
                            backoff::ExponentialBackoffBuilder::new()
                                .with_initial_interval(time::Duration::from_secs(1))
                                .with_max_elapsed_time(Some(time::Duration::from_secs(5 * 60)))
                                .with_max_interval(time::Duration::from_secs(60))
                                .with_multiplier(2.0)
                                .build(),
                            || async {
                                let in_reconnect = this.in_reconnect.get();

                                if in_reconnect {
                                    this.client()
                                        .reconnect()
                                        .await
                                        .map(|_| ())
                                        .map_err(|e| backoff::Error::transient(e))
                                } else {
                                    // User called `self.disconnect_client()`, we stop reconnections.
                                    Ok(())
                                }
                            },
                        )
                        .await;

                        this.in_reconnect.set(false);

                        if let Err(_) = res {
                            obj.emit_by_name::<()>(
                                "connection-state-changed",
                                &[&MQTTyClientConnectionState::ReconnectFailure],
                            );
                        }
                    }
                }
            ));

            // Receiving the connected status, and redirecting it to the
            // property/signal logic
            let (connected_tx, connected_rx) = async_channel::bounded(1);

            client.set_connected_callback(glib::clone!(
                #[strong]
                connected_tx,
                move |_| {
                    let _ = connected_tx.send_blocking(MQTTyClientConnectionState::Connected);
                }
            ));

            // According to Paho, this callback only works for MQTT v5, for when
            // the server is the one who disconnects.
            client.set_disconnected_callback(glib::clone!(
                #[strong]
                connected_tx,
                move |_, _, rc| {
                    let state = if rc == paho::ReasonCode::SessionTakenOver {
                        MQTTyClientConnectionState::SessionTakenOver
                    } else {
                        MQTTyClientConnectionState::Disconnected
                    };
                    let _ = connected_tx.send_blocking(state);
                }
            ));

            glib::spawn_future_local(glib::clone!(
                #[strong]
                weak_this,
                async move {
                    loop {
                        let Ok(state) = connected_rx.recv().await else {
                            return;
                        };

                        // Upgrading after receiving async channel.
                        let up = weak_this.upgrade();
                        let Some(obj) = up.as_ref().map(|this| this.obj()) else {
                            return;
                        };

                        obj.emit_by_name::<()>("connection-state-changed", &[&state]);
                    }
                }
            ));

            // Receiving message signal and redirecting it to Object signal emission
            let (message_tx, message_rx) = async_channel::bounded(1);

            client.set_message_callback(move |_, msg| {
                let Some(msg) = msg else {
                    return;
                };
                let _ = message_tx.send_blocking(msg);
            });

            glib::spawn_future_local(glib::clone!(
                #[strong]
                weak_this,
                async move {
                    loop {
                        let Ok(msg) = message_rx.recv().await else {
                            return;
                        };

                        // Upgrading after receiving async channel
                        let up = weak_this.upgrade();
                        let Some(obj) = up.as_ref().map(|this| this.obj()) else {
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
                        out_msg.set_timestamp(chrono::Local::now().to_rfc3339());

                        obj.emit_by_name::<()>("message", &[&out_msg]);
                    }
                }
            ));

            self.client.set(client).ok().unwrap();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
                vec![
                    Signal::builder("message")
                        .param_types([MQTTyClientMessage::static_type()])
                        .build(),
                    Signal::builder("connection-state-changed")
                        .param_types([MQTTyClientConnectionState::static_type()])
                        .build(),
                ]
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

            if obj.connected() {
                return Ok(());
            }

            if self.in_reconnect.get() {
                return Err("Client is automatically reconnecting".to_string());
            }

            let mqtt_version = obj.mqtt_version();

            let mut opts = paho::ConnectOptionsBuilder::with_mqtt_version(mqtt_version);

            let mut opts = opts.ssl_options(Default::default());

            opts = if let Some(username) = obj.username() {
                opts.user_name(username)
            } else {
                opts
            };

            opts = if let Some(password) = obj.password() {
                opts.password(password)
            } else {
                opts
            };

            let mut opts = opts.finalize();

            let clean_start = obj.clean_start();

            opts.set_clean_start(clean_start);
            opts.set_clean_session(clean_start);

            let res = client
                .connect(Some(opts))
                .await
                .map(|res| println!("CONNECTION SERVER RESPONSE: {res:?}"))
                .map_err(|e| e.to_string());

            res
        }

        pub async fn disconnect_client(&self) -> Result<(), String> {
            let client = self.client();

            let obj = self.obj();

            if !self.in_reconnect.get() && !obj.connected() {
                return Ok(());
            }

            self.in_reconnect.set(false);

            let res = client
                .disconnect(None)
                .await
                .map(|res| println!("DISCONNECTION SERVER RESPONSE: {res:?}"))
                .map_err(|e| e.to_string());

            // Above call can only fail if the client was disconnected, at any other
            // case, it will just disconnect, so no need to check `res`.
            obj.emit_by_name::<()>(
                "connection-state-changed",
                &[&MQTTyClientConnectionState::Disconnected],
            );

            res
        }

        pub async fn publish(&self, message: &MQTTyClientMessage) -> Result<(), String> {
            let client = self.client();

            client
                .publish(paho::Message::from(message))
                .await
                .map_err(|e| e.to_string())
        }

        pub async fn subscribe_many(
            &self,
            topics_qoss: &[(String, MQTTyClientQos)],
        ) -> Result<(), String> {
            if topics_qoss.len() == 0 {
                return Err("SUBSCRIBE protocol violation: 0 topics subscribed".to_string());
            }

            let client = self.client();

            client
                .subscribe_many(
                    topics_qoss
                        .iter()
                        .map(|t| t.0.as_str())
                        .collect::<Vec<_>>()
                        .as_slice(),
                    topics_qoss
                        .iter()
                        .map(|t| t.1)
                        .collect::<Vec<_>>()
                        .as_slice(),
                )
                .await
                .map(|res| println!("SUBSCRIPTION SERVER RESPONSE: {res:?}"))
                .map_err(|e| e.to_string())
        }

        pub async fn unsubscribe_many(&self, topics: &[String]) -> Result<(), String> {
            if topics.len() == 0 {
                return Err("UNSUBSCRIBE protocol violation: 0 topics unsubscribed".to_string());
            }

            let client = self.client();

            client
                .unsubscribe_many(topics)
                .await
                .map(|res| println!("UNSUBSCRIPTION SERVER RESPONSE: {res:?}"))
                .map_err(|e| e.to_string())
        }

        pub async fn delete_current_session(&self, reconnect_after: bool) -> Result<(), String> {
            let obj = self.obj();
            let original_clean_start = obj.clean_start();
            let was_connected = obj.connected();

            if was_connected {
                self.disconnect_client().await?;

                if original_clean_start && reconnect_after {
                    // Was already in a clean session, session is dropped once the client is
                    // disconnected, we just reconnect and return.
                    return self.connect_client().await;
                }

                if !reconnect_after {
                    return Ok(());
                }
            }

            self.clean_start.set(true);

            self.connect_client().await?;

            self.clean_start.set(original_clean_start);

            if !reconnect_after || !original_clean_start {
                self.disconnect_client().await?;
            }
            if reconnect_after && !original_clean_start {
                // The expression !original_clean_start being true produced
                // the client to be disconnected in the block above, inside this block
                // we know it's true, so we reconnect with the original clean start.
                self.connect_client().await?;
            }

            Ok(())
        }

        fn set_connected(&self, connected: bool) {
            let obj = self.obj();

            if obj.connected() == connected {
                return;
            }

            self.connected.set(connected);
            obj.notify_connected();
        }
    }
}

glib::wrapper! {
    /// This Object works as an inteface, in case the underlying MQTT library changes,
    /// also, we are using it so that we can emit signals like "connected",
    /// "connection-error" and "message"
    pub struct MQTTyClient(ObjectSubclass<imp::MQTTyClient>);
}

pub struct MQTTyClientBuilder {
    inner: glib::object::ObjectBuilder<'static, MQTTyClient>,
}

impl MQTTyClientBuilder {
    pub fn build(self) -> MQTTyClient {
        self.inner.build()
    }

    pub fn url(self, url: &str) -> Self {
        Self {
            inner: self.inner.property("url", url),
        }
    }

    pub fn mqtt_version(self, mqtt_version: MQTTyClientVersion) -> Self {
        Self {
            inner: self.inner.property("mqtt_version", mqtt_version),
        }
    }

    pub fn username(self, username: &str) -> Self {
        Self {
            inner: self.inner.property("username", username),
        }
    }

    pub fn password(self, password: &str) -> Self {
        Self {
            inner: self.inner.property("password", password),
        }
    }

    pub fn clean_start(self, clean_start: bool) -> Self {
        Self {
            inner: self.inner.property("clean_start", clean_start),
        }
    }

    pub fn client_id(self, client_id: &str) -> Self {
        Self {
            inner: self.inner.property("client_id", client_id),
        }
    }
}

impl MQTTyClient {
    #[must_use]
    pub fn builder() -> MQTTyClientBuilder {
        MQTTyClientBuilder {
            inner: glib::Object::builder(),
        }
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

    pub async fn subscribe_many(
        &self,
        topics_qoss: &[(String, MQTTyClientQos)],
    ) -> Result<(), String> {
        self.imp().subscribe_many(topics_qoss).await
    }

    pub async fn unsubscribe_many(&self, topics: &[String]) -> Result<(), String> {
        self.imp().unsubscribe_many(topics).await
    }

    /// Deletes the current session between the client and the server,
    /// effectively deleting any subscriptions that the client had.
    /// It performs reconnection if reconnect_after is true.
    ///
    /// It is specified in the MQTT spec how to perform this.
    ///
    /// MQTT 3.1.1 specification, Section 3.1.4.2:
    ///
    /// > When a Client has determined that it has no further use for the session
    /// > it should do a final connect with CleanSession set to 1 and then disconnect.
    ///
    /// https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718030
    ///
    /// This works for both MQTT v5 and v3.x
    pub async fn delete_current_session(&self, reconnect_after: bool) -> Result<(), String> {
        self.imp().delete_current_session(reconnect_after).await
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

    pub fn connect_connection_state_changed(
        &self,
        cb: impl Fn(&Self, MQTTyClientConnectionState) + 'static,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "connection-state-changed",
            false,
            glib::closure_local!(move |o: _, state: _| cb(o, state)),
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
