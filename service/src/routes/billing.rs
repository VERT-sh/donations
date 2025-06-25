use std::{convert::Infallible, num::NonZeroU64};

use crate::{config::CONFIG, state::AxumState};
use axum::{Router, response::IntoResponse, routing::post};
use macros::{IntoResponse, handler};
use stripe::Currency;
use thiserror::Error;

#[derive(Debug, Error, IntoResponse)]
enum BillingError {
    #[error("{0}")]
    #[response(hidden = true)]
    StripeError(#[from] stripe::StripeError),
    #[error("An error occurred while processing the billing request.")]
    MissingClientSecret,
    #[response(code = BAD_REQUEST)]
    #[error("Body must be a valid non-zero uint64.")]
    InvalidAmount,
}

#[handler]
async fn billing(amount: String) -> Result<String, BillingError> {
    let amount = amount
        .parse::<NonZeroU64>()
        .map_err(|_| BillingError::InvalidAmount)?;

    let amount = amount
        .get()
        .try_into()
        .map_err(|_| BillingError::InvalidAmount)?;

    let stripe = stripe::Client::new(&CONFIG.stripe.secret_key);
    let mut opts = stripe::CreatePaymentIntent::new(amount, Currency::USD);
    opts.automatic_payment_methods = Some(stripe::CreatePaymentIntentAutomaticPaymentMethods {
        enabled: true,
        ..Default::default()
    });

    let secret = stripe::PaymentIntent::create(&stripe, opts)
        .await?
        .client_secret
        .ok_or(BillingError::MissingClientSecret)?;

    Ok(secret)
}

pub fn router() -> Router<AxumState> {
    Router::new().route("/", post(billing))
}
