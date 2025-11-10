use std::cell::{Cell, OnceCell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::LazyLock;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::Signal;
use gtk::{gio, glib};

use crate::client::MQTTyClient;
use crate::client::MQTTyClientSubscription;
use crate::client::{MQTTyClientMessage, MQTTyClientVersion};

use super::models::ClientWrapperConnectionModel;
use super::store;
use super::store::MQTTySubscriptionMessagesStore;

mod imp {

    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::MQTTySubscriptionMessagesClientWrapper)]
    pub struct MQTTySubscriptionMessagesClientWrapper {
        #[property(get)]
        pub connected: Cell<bool>,

        #[property(name = "name", get, set, member = name, type = String)]
        #[property(name = "client-id", get, set, member = client_id, type = String)]
        #[property(name = "url", get, set, member = url, type = String)]
        #[property(name = "username", get, set, member = username, type = Option<String>, nullable)]
        #[property(name = "password", get, set, member = password, type = Option<String>, nullable)]
        #[property(name = "mqtt-version", get, set, member = mqtt_version, type = MQTTyClientVersion, builder(Default::default()))]
        #[property(name = "user-connected", get, set, member = user_connected, type = bool)]
        #[property(name = "wipe-queue-on-connect", get, set, member = wipe_queue_on_connect, type = bool)]
        pub connection_model: RefCell<ClientWrapperConnectionModel>,

        /** type: gio::ListStore<MQTTyClientSubscription> */
        #[property(get = |o| Self::subscriptions(o).upcast::<gio::ListModel>(), type = gio::ListModel)]
        subscriptions: OnceCell<gio::ListStore>,

        /// Maps subscriptions to tuples of indexes in ":subscriptions" prop and subscription's
        /// "::user-changed" signal handler id. When the subscription gets removed by calling
        /// self.remove_subscription(), this wrapper should both disconnect from the signal and
        /// remove the subscription from the ":subscriptions" list.
        subscriptions_map:
            RefCell<HashMap<MQTTyClientSubscription, (usize, glib::SignalHandlerId)>>,

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
                vec![Signal::builder("message")
                    .param_types([MQTTyClientMessage::static_type()])
                    .build()]
            });
            &*SIGNALS
        }
    }

    impl MQTTySubscriptionMessagesClientWrapper {
        fn priv_contains_subscription(&self, sub: &MQTTyClientSubscription) -> bool {
            let map = self.subscriptions_map.borrow();

            map.contains_key(sub) || self.contains_subscription(&sub.topic_filter())
        }

        pub fn contains_subscription(&self, topic_filter: &str) -> bool {
            let map = self.subscriptions_map.borrow();

            map.keys().any(|other| other.topic_filter() == topic_filter)
        }

        pub async fn add_subscriptions(
            &self,
            subs: &[MQTTyClientSubscription],
        ) -> Result<(), String> {
            let mut map = self.subscriptions_map.borrow_mut();

            let client = self.client();

            let list_subs = self.subscriptions();
            let n_subs = list_subs.n_items();

            let mut new_subscriptions = vec![];

            for (i, sub) in subs.iter().enumerate() {
                if self.priv_contains_subscription(sub) {
                    // If sub is contained, it means that this client wrapper
                    // is already listening to "::user-changed" and reacting
                    // accordingly to prop changes. Even though this could be a different
                    // glib::Object, the topic_filter is already handled.
                    //
                    // We continue with the next one.
                    continue;
                }

                // The sub is new and not already handled by this client.

                let prev_topic_filter = RefCell::new(sub.topic_filter());

                let new_index = (n_subs as usize) + i;

                let signal_id = sub.connect_user_changed(glib::clone!(
                    #[weak]
                    client,
                    move |sub| {
                        let cur = sub.topic_filter();
                        let prev = prev_topic_filter.replace(cur.clone());

                        let qos = sub.qos();

                        if cur != prev {
                            client.unsubscribe(prev.as_str());
                        }

                        if sub.subscribed() {
                            glib::spawn_future_local(async move {
                                client.subscribe_many(&[(cur, qos)]).await;
                            });
                        }
                    }
                ));

                map.insert(sub.clone(), (new_index, signal_id));

                new_subscriptions.push(sub.clone());
            }

            if new_subscriptions.len() > 0 {
                list_subs.splice(n_subs, 0, new_subscriptions.as_slice());

                client
                    .subscribe_many(
                        new_subscriptions
                            .iter()
                            .filter(|sub| sub.subscribed())
                            .map(|sub| (sub.topic_filter(), sub.qos()))
                            .collect::<Vec<_>>()
                            .as_slice(),
                    )
                    .await?;
            }

            Ok(())
        }

        pub async fn remove_subscription(
            &self,
            sub: &MQTTyClientSubscription,
        ) -> Result<(), String> {
            let mut map = self.subscriptions_map.borrow_mut();

            let Some((index, signal_id)) = map.remove(sub) else {
                return Ok(());
            };

            let obj = self.obj();

            obj.disconnect(signal_id);
            self.subscriptions().remove(index as u32);

            let client = self.client();
            client.unsubscribe(sub.topic_filter().as_str()).await
        }

        pub async fn resubscribe_client(&self) -> Result<(), String> {
            let obj = self.obj();
            let user_connected = obj.user_connected();

            if !user_connected {
                return Err("Cannot resubscribe client because it's disconnected".to_string());
            }

            let client = self.client();

            client.delete_current_session(user_connected).await?;

            client
                .subscribe_many(
                    obj.subscriptions()
                        .into_iter()
                        .map(|s| s.unwrap().downcast::<MQTTyClientSubscription>().unwrap())
                        .filter(|s| s.subscribed())
                        .map(|s| (s.topic_filter(), s.qos()))
                        .collect::<Vec<_>>()
                        .as_slice(),
                )
                .await
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
                if !obj.wipe_queue_on_connect() {
                    // User will receive queued messages when it connects.

                    client.connect_client().await?;
                } else {
                    // User doesn't want to receive queued messages on connect.

                    self.resubscribe_client().await?;
                }

                Ok(())
            } else {
                client.disconnect_client().await
            }
        }

        pub fn update_connection_model(&self) {
            let obj = self.obj();

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
                    let Ok(()) = store.store_message(
                        &store::ConnectionModel {
                            url: client.url(),
                            client_id: client.client_id(),
                        },
                        msg,
                    ) else {
                        return;
                    };

                    obj.emit_by_name::<()>("message", &[msg]);
                }
            ));

            // TODO: Get the previous client and disconnect gracefully.
            //
            // Currently it just drops it, which also drops the signal listeners,
            // so there is no worry with receiving messages from the old client,
            // as it is not going to happen.
            //
            // Would need this method to be async, so I don't know if it's worthful.
            self.client.borrow_mut().replace(client);
        }

        fn store(&self) -> &Rc<MQTTySubscriptionMessagesStore> {
            self.store.get().unwrap()
        }

        fn subscriptions(&self) -> gio::ListStore {
            self.subscriptions
                .get_or_init(|| gio::ListStore::new::<MQTTyClientSubscription>())
                .clone()
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
    /// client data and subscriptions, in the form of MQTTyClientConnection
    /// and MQTTyClientSubscription structs, and redirects the ":connected"
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
        conn: &ClientWrapperConnectionModel,
        store: Rc<MQTTySubscriptionMessagesStore>,
    ) -> Self {
        let o: Self = glib::Object::new();

        let im = o.imp();

        *im.connection_model.borrow_mut() = conn.clone();

        im.store.set(store);

        o.update_connection_model();

        o
    }

    /// This updates the wrapped client with the new props,
    /// it disconnects and drops the previous client and creates a new one
    /// which can be connected/disconnected by setting ":user-connected"
    /// and then calling self.sync_user_connected()
    pub fn update_connection_model(&self) {
        self.imp().update_connection_model();
    }

    pub fn contains_subscription(&self, topic_filter: &str) -> bool {
        self.imp().contains_subscription(topic_filter)
    }

    pub async fn add_subscriptions(&self, subs: &[MQTTyClientSubscription]) -> Result<(), String> {
        self.imp().add_subscriptions(subs).await
    }

    pub async fn remove_subscription(&self, sub: &MQTTyClientSubscription) -> Result<(), String> {
        self.imp().remove_subscription(sub).await
    }

    pub async fn resubscribe_client(&self) -> Result<(), String> {
        self.imp().resubscribe_client().await
    }

    /// It connects/disconnects the client depending on ":user-connected" prop
    pub async fn sync_user_connected(&self) -> Result<(), String> {
        self.imp().sync_user_connected().await
    }
}
