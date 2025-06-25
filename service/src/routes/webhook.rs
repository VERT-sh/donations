use crate::{config::CONFIG, state::AxumState};
use axum::{Router, http::HeaderMap, response::IntoResponse, routing::post};
use discord_webhook2::{message, webhook::DiscordWebhook};
use jiff::Timestamp;
use macros::{IntoResponse, handler};
use thiserror::Error;
use tracing::{Span, instrument};

#[derive(Debug, Error, IntoResponse)]
enum WebhookError {
    #[error("missing stripe signature")]
    MissingStripeSignature,
    #[error("invalid stripe signature")]
    InvalidStripeSignature,
}

#[handler]
async fn webhook(headers: HeaderMap, body: String) -> Result<(), WebhookError> {
    let signature = headers
        .get("stripe-signature")
        .ok_or(WebhookError::MissingStripeSignature)?
        .to_str()
        .map_err(|_| WebhookError::InvalidStripeSignature)?;

    let event = stripe::Webhook::construct_event(&body, signature, &CONFIG.stripe.webhook_secret)
        .map_err(|_| WebhookError::InvalidStripeSignature)?;

    tokio::spawn(handle_event(event));

    Ok(())
}

#[instrument(skip(event), fields(event_type = %event.type_))]
async fn handle_event(event: stripe::Event) {
    let span = Span::current();
    if event.type_ == stripe::EventType::PaymentIntentSucceeded {
        let stripe::EventObject::PaymentIntent(intent) = event.data.object else {
            return;
        };

        span.record("amount", format!("${:.2}", intent.amount as f64 / 100.0));

        tokio::spawn(async move {
            if let Err(e) = send_off_webhook(intent.amount as f64 / 100.0, !event.livemode).await {
                tracing::error!("failed to send webhook: {}", e);
            }
        });
    }

    if let Ok(created) = Timestamp::from_second(event.created) {
        span.record("created", created.to_string());
    }

    tracing::info!("handled webhook event");
}

pub fn router() -> Router<AxumState> {
    Router::new().route("/", post(webhook))
}

async fn send_off_webhook(amount: f64, is_test: bool) -> anyhow::Result<()> {
    let client_url = &CONFIG.webhook.url;
    let client = DiscordWebhook::new(client_url)?;
    let message = message::Message::new(|m| {
        m.content("<@&1306307808303644703> <@&1360972604579647574> <@&1339306835928158381>")
            .embed(|e| {
                let mut embed = e
                    .title("💸 new donation 💸")
                    .field(|f| f.name("amount").value(format!("${:.2}", amount)))
                    .color(0xff83fa);

                if is_test {
                    embed =
                        embed.field(|f| f.name("**THIS IS A TEST**").value("**DO NOT FREAK OUT**"))
                }

                embed
            })
    });

    client.send(&message).await?;

    Ok(())
}
