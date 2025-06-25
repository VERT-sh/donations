#![allow(dead_code)]

use std::sync::LazyLock;

use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub stripe: StripeConfig,
    pub webhook: WebhookConfig,
}

#[derive(Deserialize)]
pub struct StripeConfig {
    pub secret_key: String,
    pub webhook_secret: String,
}

#[derive(Deserialize)]
pub struct WebhookConfig {
    pub url: String,
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    Figment::new()
        .merge(Toml::file("config.toml"))
        .merge(Env::raw().split("__"))
        .extract()
        .expect("failed to read or parse config")
});

pub fn init() {
    std::hint::black_box(&*CONFIG);
}
