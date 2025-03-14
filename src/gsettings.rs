use derive_builder::Builder;
use gtk::glib;
use gtk::glib::variant::{FromVariant, StaticVariantType};

/// OpenConnections represents the 'open-connections' key GSetting, is an array
/// of tuples, where each tuple represents a single OpenConnection.
///
/// This exact struct represents a singular of those tuples.
//
// Indexes mapping:
// - 0 <-> url: URL connection
// - 1 <-> topic: MQTT topic
#[derive(Builder)]
pub struct OpenConnection {
    #[allow(unused)]
    url: String,

    #[allow(unused)]
    topic: String,
}

impl OpenConnection {
    pub fn builder() -> OpenConnectionBuilder {
        OpenConnectionBuilder::create_empty()
    }
}

impl StaticVariantType for OpenConnection {
    fn static_variant_type() -> std::borrow::Cow<'static, gtk::glib::VariantTy> {
        glib::VariantTy::TUPLE.into()
    }
}

impl FromVariant for OpenConnection {
    fn from_variant(variant: &gtk::glib::Variant) -> Option<Self> {
        let tuple = variant.get::<(String, String, String)>();
        if tuple.is_none() {
            tracing::error!("Could not get 'open-connections' in the format 'a(ss)'");
        }
        tuple.map(|tuple| Self::builder().url(tuple.0).topic(tuple.1).build().unwrap())
    }
}
