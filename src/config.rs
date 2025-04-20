use std::collections::HashMap;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use tracing::warn;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub addr: String,
    pub apprise: Apprise,
}

/// See https://github.com/caronc/apprise-api
///
/// If you use a stateless endpoint (`/notify`), then you can add the target
/// urls in `stateless_urls`.
///
/// If you use a stateful endpoint (`/notify/example`), then `stateless_urls`
/// is ignored.
///
/// You can also set additional `headers` for the Apprise request (like an
/// authentication token, for example).
#[derive(Debug, Deserialize, Clone)]
pub struct Apprise {
    pub headers: HashMap<String, String>,
    pub url: String,
    pub stateless_urls: Option<String>,
}

impl Config {
    pub fn load() -> Result<Config> {
        let cfg_path = std::env::var("FEEDBACK_CONFIG_PATH")
            .unwrap_or("config".to_string());

        let raw = config::Config::builder()
            .add_source(config::File::with_name(&cfg_path))
            .add_source(config::Environment::with_prefix("FEEDBACK"))
            .set_default("addr", "127.0.0.1:80")?
            .set_default("apprise.headers", HashMap::<String, String>::new())?
            .build()
            .context("failed to load config file")?;

        let mut cfg: Config = raw
            .try_deserialize()
            .context("failed to parse config file")?;

        let is_stateless = cfg.apprise.url.ends_with("/notify");

        if is_stateless && cfg.apprise.stateless_urls.is_none() {
            return Err(anyhow!("apprise.stateless_urls must be set when using a stateless endpoint"));
        }

        if !is_stateless && cfg.apprise.stateless_urls.is_some() {
            warn!("apprise.stateless_urls ignored when using a stateful endpoint");
            cfg.apprise.stateless_urls = None;
        }

        Ok(cfg)
    }
}
