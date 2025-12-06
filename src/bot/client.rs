use anyhow::Context;
use reqwest::{Client as HttpClient, Url, multipart::Form};

use crate::config::Config;

pub struct Client {
    base_url: Url,
    http_client: HttpClient,
}

impl Client {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            base_url: format!("https://api.telegram.org/bot{}/", config.auth.bot_token)
                .parse()
                .context("Failed to create telegram api base url")?,
            http_client: HttpClient::builder()
                .user_agent("Anon bot")
                .build()
                .context("Failed to create telegram http client")?,
        })
    }

    pub async fn setup(&self, config: &Config) -> anyhow::Result<()> {
        let url = self
            .base_url
            .join("setWebhook")
            .context("Failed to create setup url")?;

        let mut body = Form::new()
            .text(
                "url",
                format!(
                    "https://{}:{}/update",
                    config.http.public_ip, config.http.port
                ),
            )
            .file("certificate", &config.http.tls.cert)
            .await?;

        if let Some(api_token) = config.auth.api_token.clone() {
            body = body.text("secret_token", api_token);
        }

        self.http_client
            .post(url)
            .multipart(body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
