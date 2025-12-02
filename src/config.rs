use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub token: String,
    pub http: HttpConfig,
}

impl Config {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let cfg = config::Config::builder()
            .add_source(config::File::from(path))
            .build()?
            .try_deserialize()?;

        Ok(cfg)
    }
}

#[derive(Deserialize)]
pub struct HttpConfig {
    port: u16,
}
