use std::path::Path;

use serde::{Deserialize, Deserializer};
use slog::Level;
use std::{path::PathBuf, str::FromStr};

#[derive(Deserialize)]
pub struct Config {
    pub auth: AuthConfig,
    pub http: HttpConfig,
    pub log: LoggingConfig,
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
pub struct AuthConfig {
    pub bot_token: String,
    pub api_token: Option<String>,
}

#[derive(Deserialize)]
pub struct HttpConfig {
    pub port: u16,
}

#[derive(Deserialize)]
pub struct LoggingConfig {
    #[serde(deserialize_with = "deserialize_level")]
    pub level: Level,
    pub file: Option<PathBuf>,
}

fn deserialize_level<'de, D>(deserializer: D) -> Result<Level, D::Error>
where
    D: Deserializer<'de>,
{
    struct LevelVisitor;

    impl<'de> serde::de::Visitor<'de> for LevelVisitor {
        type Value = Level;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a level")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Level::from_str(v.to_uppercase().as_str())
                .map_err(|_| E::custom(format!("Failed to get level from {v}")))
        }
    }

    deserializer.deserialize_str(LevelVisitor)
}
