use crate::client::MQTTyClientVersion;

#[derive(Default, Clone)]
pub struct ClientWrapperConnectionModel {
    pub name: String,
    pub client_id: String,
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub mqtt_version: MQTTyClientVersion,
    /// The user choice for this client to be connected.
    pub user_connected: bool,

    /// This field is an abstraction of the clean_start flag in a MQTT client,
    /// designed specifically for this application.
    ///
    /// ## Explanation:
    ///
    /// If the user doesn't want to receive old messages after connecting and it was
    /// purposefully disconnected (by fully closing the app or setting connected to `false`,
    /// it doesn't count if it was because connection lost), he can set this field to
    /// `true` and when the client wrapper gets connected, it will first wipe the
    /// current session, and then finally reconnect with clean_start set always to `false`.
    ///
    /// The previously explained process is performed when the client wrapper
    /// gets constructed with this field set to `false` or when
    /// `client_wrapper.set_connected(true)` is called. You can always change
    /// this behaviour by calling `client_wrapper.update_connection_model(...)` and
    /// changing this field to other value.
    pub wipe_queue_on_connect: bool,
}
