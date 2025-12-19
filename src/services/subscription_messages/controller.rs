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

use std::cell::{Cell, OnceCell};
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::models::MQTTyConnectionSessionModel;

use super::store::MQTTySubscriptionMessagesStore;
use super::MQTTySubscriptionMessagesClientWrapper;

mod imp {

    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::MQTTySubscriptionMessagesController)]
    pub struct MQTTySubscriptionMessagesController {
        /*
         * type: gio::ListStore<MQTTySubscriptionMessagesClientWrapper>
         */
        #[property(get = Self::clients)]
        clients: OnceCell<gio::ListStore>,

        started: Cell<bool>,

        pub store: OnceCell<Rc<MQTTySubscriptionMessagesStore>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTySubscriptionMessagesController {
        const NAME: &'static str = "MQTTySubscriptionMessagesController";

        type Type = super::MQTTySubscriptionMessagesController;

        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for MQTTySubscriptionMessagesController {}

    impl MQTTySubscriptionMessagesController {
        pub fn add_clients(&self, clients: &[MQTTyConnectionSessionModel]) {
            let mut new_clients = vec![];
            let store = self.store();

            let clients_list = self.clients();

            for client in clients.iter() {
                if self.contains_connection(&client.as_ref().url, &client.client_id) {
                    // The controller already contains this connection, if you
                    // would like to show this to the user, you should call
                    // self.contains_connection(url, client_id)
                    //
                    // We continue.
                    continue;
                }

                // Is a new connection

                let wrapper = MQTTySubscriptionMessagesClientWrapper::new(client, store.clone());

                new_clients.push(wrapper);
            }

            clients_list.splice(clients_list.n_items(), 0, new_clients.as_slice());
        }

        pub async fn remove_client(
            &self,
            client: &MQTTySubscriptionMessagesClientWrapper,
        ) -> Result<(), String> {
            let clients_list = self.clients();
            let Some(index) = clients_list.find(client) else {
                return Err("Client already removed from this controller".to_string());
            };

            let res = client.disconnect_client().await;

            clients_list.remove(index as u32);

            res
        }

        pub fn contains_connection(&self, url: &str, client_id: &str) -> bool {
            let clients = self.clients_vec();

            clients
                .into_iter()
                .any(|c| c.url() == url && c.client_id() == client_id)
        }

        pub fn contains_connection_for_update(
            &self,
            old: &MQTTySubscriptionMessagesClientWrapper,
            new_url: &str,
            new_client_id: &str,
        ) -> bool {
            let clients = self.clients_vec();

            clients
                .into_iter()
                .filter(|c| c != old)
                .any(|c| c.url() == new_url && c.client_id() == new_client_id)
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
                .get_or_init(|| gio::ListStore::new::<MQTTySubscriptionMessagesClientWrapper>())
                .clone()
        }

        fn clients_vec(&self) -> Vec<MQTTySubscriptionMessagesClientWrapper> {
            self.clients()
                .iter::<_>()
                .map(|c| c.unwrap())
                .collect::<Vec<_>>()
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

        let _ = im.store.set(Rc::new(store));

        Ok(o)
    }

    pub fn add_clients(&self, clients_model: &[MQTTyConnectionSessionModel]) {
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

    pub fn contains_connection_for_update(
        &self,
        old: &MQTTySubscriptionMessagesClientWrapper,
        new_url: &str,
        new_client_id: &str,
    ) -> bool {
        self.imp()
            .contains_connection_for_update(old, new_url, new_client_id)
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
