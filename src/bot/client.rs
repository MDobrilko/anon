use anyhow::Context;
use reqwest::{Client as HttpClient, Response, Url, multipart::Form};

use crate::{config::Config, log::error};

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

        let response = self.http_client.post(url).multipart(body).send().await?;

        if let Err(err) = response.error_for_status_ref() {
            let resp_body = response.text().await.ok();
            error!(
                "Setup request failed: {}",
                resp_body.as_deref().unwrap_or("N/A")
            );

            return Err(err.into());
        }

        Ok(())
    }

    pub async fn send_message(&self, payload: &impl serde::Serialize) {
        self.send_silent_json_request("sendMessage", Some(payload))
            .await
    }

    pub async fn answer_callback_query(&self, query_id: &str, text: Option<&str>) {
        self.send_silent_json_request(
            "answerCallbackQuery",
            Some(&serde_json::json!({
                "callback_query_id": query_id,
                "text": text,
            })),
        )
        .await;
    }

    async fn send_silent_json_request<T: serde::Serialize>(
        &self,
        method: &str,
        payload: Option<&T>,
    ) {
        match self.send_json_request(method, payload).await {
            Ok(_) => {}
            Err(err) => {
                error!("{err:#}");
            }
        }
    }

    async fn send_json_request<T: serde::Serialize>(
        &self,
        method: &str,
        payload: Option<&T>,
    ) -> anyhow::Result<Response> {
        let url = self
            .base_url
            .join(method)
            .context("Failed to create sendMessage url")?;

        let mut request = self.http_client.post(url);
        if let Some(payload) = payload {
            request = request.json(&payload);
        }

        let response = request.send().await?;

        if let Err(err) = response.error_for_status_ref() {
            let resp_body = response.text().await.ok();

            return Err(err).with_context(move || {
                format!(
                    "Send \"{method}\" request failed: {}",
                    resp_body.as_deref().unwrap_or("N/A")
                )
            });
        }

        Ok(response)
    }
}
