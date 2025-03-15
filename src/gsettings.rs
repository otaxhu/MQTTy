use gtk::glib;
use gtk::glib::variant::{FromVariant, StaticVariantType};

pub struct MQTTyOpenConnection {
    pub host: String,
    pub topic: String,
}

impl MQTTyOpenConnection {
    pub fn new(host: &String, topic: &String) -> Self {
        Self {
            host: host.clone(),
            topic: topic.clone(),
        }
    }
}

impl StaticVariantType for MQTTyOpenConnection {
    fn static_variant_type() -> std::borrow::Cow<'static, gtk::glib::VariantTy> {
        glib::VariantTy::TUPLE.into()
    }
}

impl FromVariant for MQTTyOpenConnection {
    fn from_variant(variant: &gtk::glib::Variant) -> Option<Self> {
        let tuple = variant.get::<(String, String, String)>();
        if tuple.is_none() {
            tracing::error!("Could not get 'open-connections' in the format 'a(ss)'");
        }

        // Indexes mapping:
        // - 0 <-> url: URL connection
        // - 1 <-> topic: MQTT topic
        tuple.map(|tuple| Self::new(&tuple.0, &tuple.1))
    }
}
