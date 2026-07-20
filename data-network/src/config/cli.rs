use std::net::IpAddr;
use std::{collections::HashMap, path::PathBuf};

use figment::{providers::Serialized, value::Value};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::config::types::TrustOptions;

/// CLI configuration overrides.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CliConfig {
    pub execution_rpc: Option<Url>,
    pub verifiable_api: Option<Url>,
    pub consensus_rpc: Option<Url>,
    pub trust_options: Option<TrustOptions>,
    pub rpc_bind_ip: Option<IpAddr>,
    pub rpc_port: Option<u16>,
    pub data_dir: Option<PathBuf>,
}

impl CliConfig {
    pub fn as_provider(&self, network: &str) -> Serialized<HashMap<&str, Value>> {
        let mut user_dict = HashMap::new();

        if let Some(rpc) = &self.execution_rpc {
            user_dict.insert("execution_rpc", Value::from(rpc.to_string()));
        }

        if let Some(api) = &self.verifiable_api {
            user_dict.insert("verifiable_api", Value::from(api.to_string()));
        }

        if let Some(rpc) = &self.consensus_rpc {
            user_dict.insert("consensus_rpc", Value::from(rpc.to_string()));
        }

        if let Some(trust_options) = &self.trust_options {
            user_dict.insert(
                "trust_options",
                Value::serialize(trust_options).expect("trust options should serialize"),
            );
        }

        if let Some(ip) = self.rpc_bind_ip {
            user_dict.insert("rpc_bind_ip", Value::from(ip.to_string()));
        }

        if let Some(port) = self.rpc_port {
            user_dict.insert("rpc_port", Value::from(port));
        }

        if let Some(data_dir) = self.data_dir.as_ref() {
            user_dict.insert("data_dir", Value::from(data_dir.to_str().unwrap()));
        }

        Serialized::from(user_dict, network)
    }
}
