use alloy::primitives::Address;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// Address label store for mapping addresses to human-readable names
#[derive(Debug, Clone)]
pub struct LabelStore {
    labels: HashMap<Address, String>,
}

impl LabelStore {
    /// Create an empty label store
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
        }
    }

    /// Load labels from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> eyre::Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::load_from_json(&content)
    }

    /// Load labels from JSON string
    pub fn load_from_json(json: &str) -> eyre::Result<Self> {
        let value: Value = serde_json::from_str(json)?;
        let mut labels = HashMap::new();

        if let Value::Object(map) = value {
            for (address_str, label_value) in map {
                if let Value::String(label) = label_value {
                    // Handle addresses with or without checksum
                    let normalized = address_str.to_lowercase();
                    if let Ok(address) = Address::from_str(&normalized) {
                        labels.insert(address, label);
                    }
                }
            }
        }

        Ok(Self { labels })
    }

    /// Load labels with embedded defaults
    pub fn load_with_defaults() -> Self {
        // Try to load from file first
        let data_paths = [
            "data/labels.json",
            "./data/labels.json",
            "../data/labels.json",
        ];

        for path in data_paths {
            if let Ok(store) = Self::load_from_file(path) {
                tracing::info!("Loaded {} address labels from {}", store.labels.len(), path);
                return store;
            }
        }

        // Fall back to embedded defaults
        let default_json = include_str!("../data/labels.json");
        match Self::load_from_json(default_json) {
            Ok(store) => {
                tracing::info!(
                    "Loaded {} address labels from embedded defaults",
                    store.labels.len()
                );
                store
            }
            Err(e) => {
                tracing::warn!("Failed to load labels: {}, using empty store", e);
                Self::new()
            }
        }
    }

    /// Get the label for an address
    pub fn get(&self, address: &Address) -> Option<String> {
        self.labels.get(address).cloned()
    }

    /// Check if an address has a label
    pub fn has_label(&self, address: &Address) -> bool {
        self.labels.contains_key(address)
    }

    /// Get the total number of labels
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }

    /// Add a label for an address
    pub fn insert(&mut self, address: Address, label: String) {
        self.labels.insert(address, label);
    }
}

impl Default for LabelStore {
    fn default() -> Self {
        Self::load_with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_json() {
        let json = r#"{
            "0x28C6c06298d514Db089934071355E5743bf21d60": "Binance",
            "0x71660c4005BA85c37ccec55d0C4493E66Fe775d3": "Coinbase"
        }"#;

        let store = LabelStore::load_from_json(json).unwrap();
        assert_eq!(store.len(), 2);

        let binance_addr =
            Address::from_str("0x28C6c06298d514Db089934071355E5743bf21d60").unwrap();
        assert_eq!(store.get(&binance_addr), Some("Binance".to_string()));
    }
}

