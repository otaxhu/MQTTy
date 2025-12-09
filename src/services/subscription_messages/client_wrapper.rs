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
use std::rc::Rc;
use std::sync::LazyLock;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::Signal;
use gtk::{gio, glib};

use crate::client::{
    MQTTyClient, MQTTyClientConnectionState, MQTTyClientMessage, MQTTyClientVersion,
};
use crate::models::{MQTTyConnectionModel, MQTTySubscriptionModel};

use super::store;
use super::store::MQTTySubscriptionMessagesStore;
use super::MQTTySubscriptionMessagesSubscription;

mod imp {

    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::MQTTySubscriptionMessagesClientWrapper)]
    pub struct MQTTySubscriptionMessagesClientWrapper {
        #[property(get)]
        pub connected: Cell<bool>,

        #[property(name = "name", get, member = name, type = String)]
        #[property(name = "client-id", get, member = client_id, type = String)]
        #[property(name = "url", get, member = url, type = String)]
        #[property(name = "username", get, member = username, type = Option<String>, nullable)]
        #[property(name = "password", get, member = password, type = Option<String>, nullable)]
        #[property(name = "mqtt-version", get, member = mqtt_version, type = MQTTyClientVersion, builder(Default::default()))]
        #[property(name = "user-connected", get, set, member = user_connected, type = bool)]
        #[property(name = "wipe-queue-on-connect", get, member = wipe_queue_on_connect, type = bool)]
        pub connection_model: RefCell<MQTTyConnectionModel>,

        /** type: gio::ListStore<MQTTySubscriptionMessagesSubscription> */
        #[property(get = Self::subscriptions)]
        subscriptions: OnceCell<gio::ListStore>,

        pub store: OnceCell<Rc<MQTTySubscriptionMessagesStore>>,
        pub client: RefCell<Option<MQTTyClient>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionMessagesClientWrapper {
        const NAME: &'static str = "MQTTySubscriptionMessagesClientWrapper";

        type Type = super::MQTTySubscriptionMessagesClientWrapper;

        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTySubscriptionMessagesClientWrapper {
        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
                vec![
                    Signal::builder("message")
                        .param_types([MQTTyClientMessage::static_type()])
                        .build(),
                    Signal::builder("connection-state-changed")
                        .param_types([MQTTyClientConnectionState::static_type()])
                        .build(),
                    Signal::builder("identity-changed").build(),
                ]
            });
            &*SIGNALS
        }
    }

    impl MQTTySubscriptionMessagesClientWrapper {
        pub fn contains_subscription(&self, topic_filter: &str) -> bool {
            let subs = self.subscriptions_vec();

            subs.into_iter().any(|s| s.topic_filter() == topic_filter)
        }

        pub fn contains_subscription_for_update(
            &self,
            old: &MQTTySubscriptionMessagesSubscription,
            new_topic_filter: &str,
        ) -> bool {
            let subs = self.subscriptions_vec();

            subs.into_iter()
                .filter(|s| s != old)
                .any(|s| s.topic_filter() == new_topic_filter)
        }

        pub async fn update_subscription(
            &self,
            old: &MQTTySubscriptionMessagesSubscription,
            new_model: &MQTTySubscriptionModel,
        ) -> (bool, Option<String>) {
            if self.contains_subscription_for_update(old, &new_model.topic_filter) {
                return (false, None);
            }

            let old_topic = old.topic_filter();

            let mut res = (true, None);

            if old_topic != new_model.topic_filter {
                let client = self.client();

                res.1 = client.unsubscribe_many(&[old_topic]).await.err();
            }

            // This emits "::notify" signal, so we let the widgets handle it.
            old.set_subscription_model(new_model);

            res
        }

        pub async fn sync_all_subscriptions(&self) -> Result<(), String> {
            let subs = self.subscriptions_vec();
            let client = self.client();
            let obj = self.obj();

            if obj.connected() && subs.len() > 0 {
                client
                    .unsubscribe_many(
                        subs.iter()
                            .filter(|s| !s.user_subscribed())
                            .map(|s| s.topic_filter())
                            .collect::<Vec<_>>()
                            .as_slice(),
                    )
                    .await?;

                client
                    .subscribe_many(
                        subs.iter()
                            .filter(|s| s.user_subscribed())
                            .map(|s| (s.topic_filter(), s.qos()))
                            .collect::<Vec<_>>()
                            .as_slice(),
                    )
                    .await?;
            }

            Ok(())
        }

        pub async fn sync_subscription(
            &self,
            sub: &MQTTySubscriptionMessagesSubscription,
        ) -> Result<(), String> {
            let client = self.client();

            if sub.user_subscribed() {
                client
                    .subscribe_many(&[(sub.topic_filter(), sub.qos())])
                    .await
            } else {
                client.unsubscribe_many(&[sub.topic_filter()]).await
            }
        }

        pub fn add_subscriptions(&self, subs: &[MQTTySubscriptionModel]) {
            let list_subs = self.subscriptions();

            let mut new_subscriptions = vec![];

            for sub_model in subs {
                if self.contains_subscription(&sub_model.topic_filter) {
                    continue;
                }

                let sub = MQTTySubscriptionMessagesSubscription::new(sub_model);

                new_subscriptions.push(sub);
            }

            list_subs.splice(list_subs.n_items(), 0, new_subscriptions.as_slice());
        }

        pub async fn remove_subscription(
            &self,
            sub: &MQTTySubscriptionMessagesSubscription,
        ) -> Result<(), String> {
            let list_subs = self.subscriptions();
            let Some(index) = list_subs.find(sub) else {
                return Err("Subscription already removed from this client wrapper".to_string());
            };

            let obj = self.obj();

            let mut res = Ok(());

            if obj.connected() {
                let client = self.client();
                res = client.unsubscribe_many(&[sub.topic_filter()]).await;
            }

            list_subs.remove(index);

            res
        }

        pub async fn reset_session(&self) -> Result<(), String> {
            let obj = self.obj();
            let user_connected = obj.user_connected();

            let client = self.client();

            client.delete_current_session(user_connected).await?;

            let subs = self.subscriptions_vec();

            if user_connected {
                client
                    .subscribe_many(
                        subs.into_iter()
                            .filter(|s| s.user_subscribed())
                            .map(|s| (s.topic_filter(), s.qos()))
                            .collect::<Vec<_>>()
                            .as_slice(),
                    )
                    .await?;
            }

            Ok(())
        }

        pub async fn sync_user_connected(&self) -> Result<(), String> {
            let obj = self.obj();
            let connected = obj.connected();
            let user_connected = obj.user_connected();

            if connected == user_connected {
                return Ok(());
            }

            let client = self.client();

            if user_connected {
                if obj.wipe_queue_on_connect() {
                    // User doesn't want to receive queued messages on connect.

                    self.reset_session().await
                } else {
                    // User will receive queued messages when it connects.

                    client.connect_client().await
                }
            } else {
                client.disconnect_client().await
            }
        }

        pub async fn disconnect_client(&self) -> Result<(), String> {
            self.client().disconnect_client().await
        }

        pub fn set_connection_model(&self, conn_model: &MQTTyConnectionModel) {
            let obj = self.obj();

            let identity_changed =
                obj.client_id() != conn_model.client_id || obj.url() != conn_model.url;

            *self.connection_model.borrow_mut() = conn_model.clone();

            let mut builder = MQTTyClient::builder()
                .mqtt_version(obj.mqtt_version())
                .client_id(&obj.client_id())
                .url(&obj.url())
                // clean_start always false
                .clean_start(false);

            builder = if let Some(ref username) = obj.username() {
                builder.username(username)
            } else {
                builder
            };

            builder = if let Some(ref password) = obj.password() {
                builder.password(password)
            } else {
                builder
            };

            let client = builder.build();

            let store = self.store();

            client.connect_connected_notify(glib::clone!(
                #[weak(rename_to = this)]
                self,
                #[weak]
                obj,
                move |client| {
                    this.connected.set(client.connected());
                    obj.notify_connected();
                }
            ));

            client.connect_message(glib::clone!(
                #[weak]
                obj,
                #[weak]
                store,
                move |client, msg| {
                    if let Err(e) = store.store_message(
                        &store::ConnectionModel {
                            url: client.url(),
                            client_id: client.client_id(),
                        },
                        msg,
                    ) {
                        println!("Error while storing message: {e:?}");
                    }

                    // If the message could not be stored, doesn't matter, we still
                    // send the signal.
                    obj.emit_by_name::<()>("message", &[msg]);
                }
            ));

            client.connect_connection_state_changed(glib::clone!(
                #[weak]
                obj,
                move |_, state| {
                    obj.emit_by_name::<()>("connection-state-changed", &[&state]);
                }
            ));

            // TODO: Get the previous client and disconnect gracefully.
            //
            // Currently it just drops it, which also drops the signal listeners,
            // so there is no worry with receiving messages from the old client,
            // as it is not going to happen.
            //
            // Would need this method to be async, so I don't know if it's worthful.
            self.client.borrow_mut().replace(client.clone());

            // Syncing connected
            client.notify_connected();

            let props_to_notify = [
                "name",
                "client-id",
                "url",
                "username",
                "password",
                "mqtt-version",
                "user-connected",
                "wipe-queue-on-connect",
            ];

            for prop in props_to_notify {
                obj.notify(prop);
            }

            if identity_changed {
                obj.emit_by_name::<()>("identity-changed", &[]);
            }
        }

        pub fn get_recent_messages(
            &self,
            cursor_id: Option<i64>,
            limit: i64,
        ) -> Result<(Vec<MQTTyClientMessage>, Option<i64>), store::Error> {
            let store = self.store();
            let obj = self.obj();

            store.get_recent_messages_for_connection(
                &store::ConnectionModel {
                    client_id: obj.client_id(),
                    url: obj.url(),
                },
                cursor_id,
                limit,
            )
        }

        fn store(&self) -> &Rc<MQTTySubscriptionMessagesStore> {
            self.store.get().unwrap()
        }

        fn subscriptions(&self) -> gio::ListStore {
            self.subscriptions
                .get_or_init(|| gio::ListStore::new::<MQTTySubscriptionMessagesSubscription>())
                .clone()
        }

        fn subscriptions_vec(&self) -> Vec<MQTTySubscriptionMessagesSubscription> {
            self.subscriptions()
                .iter::<_>()
                .map(|s| s.unwrap())
                .collect::<Vec<_>>()
        }

        fn client(&self) -> MQTTyClient {
            self.client.borrow().as_ref().unwrap().clone()
        }
    }
}

glib::wrapper! {
    /// This wraps a MQTTyClient, in order to store incoming messages to the
    /// MQTTySubscriptionMessagesStore, and then redirects them to "::message"
    /// signal of this object after succesfully storing them, as well as
    /// providing an API for retrieving messages from the store, getting
    /// client data and subscriptions, and redirects the ":connected"
    /// property from the underlying client to this wrapper property.
    ///
    /// This class is supposed to be used only by widgets to perform all of the
    /// operations mentioned above.
    pub struct MQTTySubscriptionMessagesClientWrapper(ObjectSubclass<imp::MQTTySubscriptionMessagesClientWrapper>);
}

impl MQTTySubscriptionMessagesClientWrapper {
    /*
     * Visibility is `pub(super)` so that it can only be constructed by the controller
     */
    pub(super) fn new(
        conn: &MQTTyConnectionModel,
        store: Rc<MQTTySubscriptionMessagesStore>,
    ) -> Self {
        let o: Self = glib::Object::new();

        let im = o.imp();

        im.store
            .set(store)
            .unwrap_or_else(|_| panic!("Store already set"));

        o.set_connection_model(conn);

        o
    }

    pub fn connection_model(&self) -> MQTTyConnectionModel {
        self.imp().connection_model.borrow().clone()
    }

    /// This updates the wrapped client with the new props,
    /// it disconnects and drops the previous client and creates a new one
    /// which can be connected/disconnected by setting ":user-connected"
    /// and then calling self.sync_user_connected()
    ///
    /// Additionally it emits all of the "::notify" signals that correspond
    /// to the connection_model.
    pub fn set_connection_model(&self, conn_model: &MQTTyConnectionModel) {
        self.imp().set_connection_model(conn_model);
    }

    pub fn contains_subscription(&self, topic_filter: &str) -> bool {
        self.imp().contains_subscription(topic_filter)
    }

    pub fn contains_subscription_for_update(
        &self,
        old: &MQTTySubscriptionMessagesSubscription,
        new_topic_filter: &str,
    ) -> bool {
        self.imp()
            .contains_subscription_for_update(old, new_topic_filter)
    }

    /// Adds subscriptions
    ///
    /// It doesn't send UN/SUBSCRIBE packets to the broker, to do that call
    /// sync_subsciption() or sync_all_subscriptions()
    pub fn add_subscriptions(&self, subs: &[MQTTySubscriptionModel]) {
        self.imp().add_subscriptions(subs);
    }

    pub async fn remove_subscription(
        &self,
        sub: &MQTTySubscriptionMessagesSubscription,
    ) -> Result<(), String> {
        self.imp().remove_subscription(sub).await
    }

    /// It resets the current session.
    ///
    /// This essentially performs:
    ///
    /// 1. Session deletion (if there is any).
    /// 2. Connects the client (if ":user-connected" was `true`).
    /// 3. And then sends the subscriptions that have ":start-subscribed" set to `true`.
    ///
    /// This causes any previous subscription to be deleted, this should be used if
    /// the user is receiving messages from a topic it is no suscribed to,
    /// so the only method they can call is this, in order to delete those subscriptions.
    pub async fn reset_session(&self) -> Result<(), String> {
        self.imp().reset_session().await
    }

    /// It connects/disconnects the client depending on ":user-connected" prop
    pub async fn sync_user_connected(&self) -> Result<(), String> {
        self.imp().sync_user_connected().await
    }

    /// Tries to update old_sub with new_model and send the corresponding
    /// UNSUBSCRIBE packet if topic filter was different than the previous one.
    ///
    /// Just like add_subscriptions(), this method doesn't send the subscriptions,
    /// to do that call sync_subscription() method.
    ///
    /// Returns (updated, unsub_err) tuple, the first tuple's entry indicates if the
    /// subscription was succesfully updated and it was not a duplicate, and the
    /// second indicates the UNSUBSCRIBE packet error if it's Some(...)
    pub async fn update_subscription(
        &self,
        old_sub: &MQTTySubscriptionMessagesSubscription,
        new_model: &MQTTySubscriptionModel,
    ) -> (bool, Option<String>) {
        self.imp().update_subscription(old_sub, new_model).await
    }

    pub async fn sync_subscription(
        &self,
        sub: &MQTTySubscriptionMessagesSubscription,
    ) -> Result<(), String> {
        self.imp().sync_subscription(sub).await
    }

    /// Use this when the client is first created and after adding all
    /// subscriptions, e.g. when the app starts.
    ///
    /// If the user adds a single subscription, prefer to use sync_subscription()
    /// method.
    pub async fn sync_all_subscriptions(&self) -> Result<(), String> {
        self.imp().sync_all_subscriptions().await
    }

    pub fn get_recent_messages(
        &self,
        cursor_id: Option<i64>,
        limit: i64,
    ) -> Result<(Vec<MQTTyClientMessage>, Option<i64>), store::Error> {
        self.imp().get_recent_messages(cursor_id, limit)
    }

    pub(super) async fn disconnect_client(&self) -> Result<(), String> {
        self.imp().disconnect_client().await
    }

    pub fn connect_message(
        &self,
        cb: impl Fn(&Self, MQTTyClientMessage) + 'static,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "message",
            false,
            glib::closure_local!(|o: _, msg: _| cb(o, msg)),
        )
    }

    pub fn connect_connection_state_changed(
        &self,
        cb: impl Fn(&Self, MQTTyClientConnectionState) + 'static,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "connection-state-changed",
            false,
            glib::closure_local!(|o: _, state: _| cb(o, state)),
        )
    }

    pub fn connect_identity_changed(&self, cb: impl Fn(&Self) + 'static) -> glib::SignalHandlerId {
        self.connect_closure(
            "identity-changed",
            false,
            glib::closure_local!(|o: _| cb(o)),
        )
    }
}
