use std::cell::{OnceCell, RefCell};
use std::sync::LazyLock;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::glib::subclass::Signal;

use crate::gsettings::MQTTySettingConnection;

mod imp {

    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::MQTTyClient)]
    pub struct MQTTyClient {
        #[property(get, set, construct)]
        settings_conn: RefCell<MQTTySettingConnection>,

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
                .server_uri(obj.settings_conn().url())
                .create_client()
            {
                Err(e) => {
                    println!("CLIENT CREATION ERROR {:?}", e);

                    // TODO: I think this is not gonna be emitted to anybody
                    obj.emit_by_name::<()>("connection-error", &[&e.to_string()]);
                    return;
                }
                Ok(c) => c,
            };

            // Receiving connected signal and redirecting it to Object signal emission
            let (connected_tx, connected_rx) = async_channel::bounded(1);

            client.set_connected_callback(move |_| {
                let _ = connected_tx.send_blocking(());
            });

            glib::spawn_future_local(glib::clone!(
                #[weak]
                obj,
                async move {
                    loop {
                        let _ = connected_rx.recv().await;
                        obj.emit_by_name::<()>("connected", &[&true]);
                    }
                }
            ));

            // Receiving disconnected signal and redirecting it to Object signal emission
            let (disconnected_tx, disconnected_rx) = async_channel::bounded(1);

            client.set_disconnected_callback(move |_, _, _| {
                let _ = disconnected_tx.send_blocking(());
            });

            glib::spawn_future_local(glib::clone!(
                #[weak]
                obj,
                async move {
                    loop {
                        let _ = disconnected_rx.recv().await;
                        obj.emit_by_name::<()>("connected", &[&false]);
                    }
                }
            ));

            // Receiving message signal and redirecting it to Object signal emission
            let (message_tx, message_rx) = async_channel::bounded(1);

            client.set_message_callback(move |_, msg| {
                let msg = msg.unwrap();
                let _ = message_tx.send_blocking(msg.payload().to_owned());
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

                        obj.emit_by_name::<()>("message", &[&glib::Variant::from(msg)]);
                    }
                }
            ));

            glib::spawn_future_local(glib::clone!(
                #[strong]
                client,
                #[weak]
                obj,
                async move {
                    let res = client.connect(None).await;

                    match res {
                        Err(e) => {
                            println!("CLIENT CONNECTION ERROR {:?}", e);
                            obj.emit_by_name::<()>("connection-error", &[]);
                        }
                        Ok(_) => {
                            let _ = client
                                .subscribe(obj.settings_conn().topic(), paho::QOS_0)
                                .await;
                        }
                    };
                }
            ));

            let _ = self.client.set(client);
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
                vec![
                    // true when it's connected, false otherwise
                    Signal::builder("connected")
                        .param_types([glib::Type::BOOL])
                        .build(),
                    Signal::builder("connection-error").build(),
                    // Variant it's a Vec<u8>, containing the message bytes
                    Signal::builder("message")
                        .param_types([glib::Type::VARIANT])
                        .build(),
                ]
            });
            &*SIGNALS
        }
    }

    impl MQTTyClient {
        pub fn disconnect_client(&self) {
            let client = self.client.get().unwrap();
            client.disconnect(None);
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
    pub fn new(conn: &MQTTySettingConnection) -> Self {
        glib::Object::builder()
            .property("settings_conn", conn)
            .build()
    }

    pub fn disconnect_client(&self) {
        self.imp().disconnect_client();
    }
}
