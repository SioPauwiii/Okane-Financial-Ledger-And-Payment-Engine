use stripe::{
    CheckoutSession, CheckoutSessionMode, CreateCheckoutSession,
    CreateCheckoutSessionLineItems, CreateCheckoutSessionLineItemsPriceData,
    CreateCheckoutSessionLineItemsPriceDataProductData, Currency,
    Client,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use crate::errors::AppError;

pub async fn create_checkout_session(
    client: &Client,
    amount: Decimal,
    source_account_number: Option<String>,
    target_account_number: Option<String>,
) -> Result<CheckoutSession, AppError> {

    // Convert Decimal (e.g. 50.00) to i64 cents (e.g. 5000) that Stripe expects
    let amount_cents = (amount * Decimal::new(100, 0))
        .to_i64()
        .ok_or_else(|| AppError::BadRequest("Invalid amount".to_string()))?;

    if amount_cents <= 0 {
        return Err(AppError::BadRequest("Amount must be greater than zero".to_string()));
    }

    // Build the metadata HashMap (Metadata is a type alias for HashMap<String, String>)
    let mut metadata = std::collections::HashMap::new();
    if let Some(source) = source_account_number {
        metadata.insert("source_account_number".to_string(), source);
    }
    if let Some(target) = target_account_number {
        metadata.insert("target_account_number".to_string(), target);
    }

    // Build all params using struct field syntax (no ::new() constructors exist)
    let params = CreateCheckoutSession {
        line_items: Some(vec![
            CreateCheckoutSessionLineItems {
                price_data: Some(CreateCheckoutSessionLineItemsPriceData {
                    currency: Currency::PHP,
                    unit_amount: Some(amount_cents),
                    product_data: Some(CreateCheckoutSessionLineItemsPriceDataProductData {
                        name: "Account Deposit".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                quantity: Some(1),
                ..Default::default()
            },
        ]),
        mode: Some(CheckoutSessionMode::Payment),
        success_url: Some("http://localhost:8081/success?session_id={CHECKOUT_SESSION_ID}"),
        cancel_url: Some("http://localhost:8081/cancel"),
        metadata: Some(metadata),
        ..Default::default()
    };

    // Send the request to Stripe and get back a session with a `url` field
    let session = CheckoutSession::create(client, params)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Stripe error: {}", e)))?;

    Ok(session)
}