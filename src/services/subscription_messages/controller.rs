use std::cell::{Cell, OnceCell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::client::MQTTyClient;

use super::models::ClientWrapperConnectionModel;
use super::store::MQTTySubscriptionMessagesStore;
use super::MQTTySubscriptionMessagesClientWrapper;

mod imp {

    use super::*;

    /// This are the same identity fields as mentioned in the store's database tables.
    /// See [store.rs](./store.rs)
    ///
    /// This is a little optimization to not perform a full Eq and Hash call on all fields
    /// in that struct, as we know that 2 clients with the same url and client_id
    /// cannot coexist per the standard spec.
    #[derive(PartialEq, Eq, Hash)]
    struct ClientsMapKey {
        client_id: String,
        url: String,
    }

    struct ClientsMapValue {
        index: usize,
    }

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::MQTTySubscriptionMessagesController)]
    pub struct MQTTySubscriptionMessagesController {
        /*
         * type: gio::ListStore<MQTTySubscriptionMessagesClientWrapper>
         */
        #[property(get = |o| Self::clients(o).upcast::<gio::ListModel>(), type = gio::ListModel)]
        clients: OnceCell<gio::ListStore>,

        started: Cell<bool>,

        pub store: OnceCell<Rc<MQTTySubscriptionMessagesStore>>,

        /// Maps url and client_id to client wrapper's indexes on ":clients" list
        clients_map: RefCell<HashMap<MQTTySubscriptionMessagesClientWrapper, usize>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionMessagesController {
        const NAME: &'static str = "MQTTySubscriptionMessagesController";

        type Type = super::MQTTySubscriptionMessagesController;

        type ParentType = glib::Object;
    }

    impl ObjectImpl for MQTTySubscriptionMessagesController {}

    impl MQTTySubscriptionMessagesController {
        pub fn add_clients(&self, clients: &[ClientWrapperConnectionModel]) {
            let mut map = self.clients_map.borrow_mut();
            let mut new_clients = vec![];
            let store = self.store();

            let clients_list = self.clients();
            let n_clients = clients_list.n_items();

            for (i, client) in clients.iter().enumerate() {
                if self.contains_connection(&client.url, &client.client_id) {
                    // The controller already contains this connection, if you
                    // would like to show this to the user, you should call
                    // self.contains_connection(url, client_id)
                    //
                    // We continue.
                    continue;
                }

                // Is a new connection

                let new_index = (n_clients as usize) + i;

                let wrapper = MQTTySubscriptionMessagesClientWrapper::new(client, store.clone());

                map.insert(wrapper.clone(), new_index);

                new_clients.push(wrapper);
            }

            clients_list.splice(n_clients, 0, new_clients.as_slice());
        }

        pub async fn remove_client(
            &self,
            client: &MQTTySubscriptionMessagesClientWrapper,
        ) -> Result<(), String> {
            let mut map = self.clients_map.borrow_mut();

            let Some(index) = map.remove(client) else {
                return Ok(());
            };

            let clients_list = self.clients();
            clients_list.remove(index as u32);

            client.set_user_connected(false);
            client.sync_user_connected().await
        }

        pub fn contains_connection(&self, url: &str, client_id: &str) -> bool {
            let map = self.clients_map.borrow();

            map.keys()
                .any(|c| c.url() == url && c.client_id() == client_id)
        }

        pub async fn start_controller(&self) -> Result<(), String> {
            let started = self.started.get();

            if started {
                return Err("controller already started".to_string());
            }
            self.started.set(true);

            let obj = self.obj();

            let clients_list = obj
                .clients()
                .iter::<MQTTySubscriptionMessagesClientWrapper>()
                .map(|c| c.unwrap())
                .collect::<Vec<_>>();

            futures::future::join_all(clients_list.iter().map(|c| c.sync_user_connected()))
                .await
                .into_iter()
                .filter(|res| res.is_err())
                .map(|res| res.unwrap_err())
                .for_each(|res| {
                    println!("subscription_messages: Error while trying to connect client: {res}")
                });

            Ok(())
        }

        fn store(&self) -> &Rc<MQTTySubscriptionMessagesStore> {
            self.store.get().unwrap()
        }

        fn clients(&self) -> gio::ListStore {
            self.clients
                .get_or_init(|| gio::ListStore::new::<MQTTyClient>())
                .clone()
        }
    }
}

glib::wrapper! {
    pub struct MQTTySubscriptionMessagesController(ObjectSubclass<imp::MQTTySubscriptionMessagesController>);
}

impl MQTTySubscriptionMessagesController {
    pub fn new() -> Result<Self, String> {
        let o: Self = glib::Object::new();

        let im = o.imp();

        let store = MQTTySubscriptionMessagesStore::new().map_err(|e| e.to_string())?;

        im.store.set(Rc::new(store));

        Ok(o)
    }

    pub fn add_clients(&self, clients_model: &[ClientWrapperConnectionModel]) {
        self.imp().add_clients(clients_model)
    }

    pub async fn remove_client(
        &self,
        client: &MQTTySubscriptionMessagesClientWrapper,
    ) -> Result<(), String> {
        self.imp().remove_client(client).await
    }

    pub fn contains_connection(&self, url: &str, client_id: &str) -> bool {
        self.imp().contains_connection(url, client_id)
    }

    /// This method is a little helper that calls `sync_user_connected()` for
    /// each client wrapper, it cannot be called more than once.
    ///
    /// Should be called after adding all the clients, putting the clients list
    /// into a gtk::ListBox, and setting up all of the signal listeners for each
    /// client wrapper.
    ///
    /// You can still add clients after started, though you would have to manually
    /// start them.
    pub async fn start_controller(&self) -> Result<(), String> {
        self.imp().start_controller().await
    }
}
