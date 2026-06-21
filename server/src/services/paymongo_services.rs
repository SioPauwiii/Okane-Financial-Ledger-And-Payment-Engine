use hmac::{Hmac, Mac, KeyInit};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use sha2::Sha256;
use serde_json::json;
use crate::errors::AppError;

type HmacSha256 = Hmac<Sha256>;

/// Converts a Decimal amount (e.g. 100.00 PHP) to centavos (e.g. 10000)
/// as PayMongo expects integer centavos, not decimal pesos.
fn convert_amount_to_centavos(amount: Decimal) -> Result<i64, AppError> {
    (amount * Decimal::new(100, 0))  // 100, scale=0 means the integer 100
        .to_i64()
        .ok_or_else(|| AppError::BadRequest("Invalid amount: could not convert to centavos".to_string()))
}

/// Parses the PayMongo signature header and returns (timestamp, te_signature).
/// Header format: "t=1716000000,te=abc123...,li=xyz..."
fn extract_timestamp_and_sig(sig_header: &str) -> Option<(String, String)> {
    let mut timestamp = "";
    let mut te_signature = "";

    for part in sig_header.split(',') {
        if let Some(val) = part.strip_prefix("t=") {
            timestamp = val;
        } else if let Some(val) = part.strip_prefix("te=") {
            te_signature = val;
        }
    }

    if timestamp.is_empty() || te_signature.is_empty() {
        return None;
    }

    Some((timestamp.to_string(), te_signature.to_string()))
}

/// Calls PayMongo's Checkout Session V2 API and returns the hosted payment URL.
pub async fn create_checkout_session(
    client: &reqwest::Client,
    secret_key: &str,
    amount: Decimal,
    account_number: &str,
    transaction_uuid: &str, // stored in metadata so the webhook can link the completed record
) -> Result<String, AppError> {
    let amount_in_centavos = convert_amount_to_centavos(amount)?;

    if amount_in_centavos <= 0 {
        return Err(AppError::BadRequest("Amount must be greater than zero".to_string()));
    }

    let body = json!({
        "data": {
            "attributes": {
                "line_items": [{
                    "currency": "PHP",
                    "amount": amount_in_centavos,
                    "name": "Okane Deposit",
                    "description": "Account deposit",
                    "quantity": 1
                }],
                "payment_method_types": ["gcash", "paymaya", "card"],
                "success_url": "http://localhost:8081/success",
                "cancel_url": "http://localhost:8081/cancel",
                "metadata": {
                    "account_number": account_number,
                    "transaction_uuid": transaction_uuid
                }
            }
        }
    });

    let response = client
        .post("https://api.paymongo.com/v2/checkout_sessions")
        .basic_auth(secret_key, None::<&str>)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::InternalServerError(format!("PayMongo request failed: {}", e)))?;

    if !response.status().is_success() {
        let err_text = response.text().await.unwrap_or_default();
        return Err(AppError::InternalServerError(format!("PayMongo error: {}", err_text)));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to parse PayMongo response: {}", e)))?;

    let checkout_url = json["data"]["attributes"]["checkout_url"]
        .as_str()
        .ok_or_else(|| AppError::InternalServerError("PayMongo did not return a checkout_url".to_string()))?
        .to_string();

    Ok(checkout_url)
}

/// Verifies the HMAC-SHA256 signature PayMongo attaches to every webhook request.
/// Returns true if valid, false if the signature doesn't match or the header is malformed.
pub fn verify_webhook_signature(raw_body: &[u8], sig_header: &str, secret: &str) -> bool {
    let (timestamp, te_signature) = match extract_timestamp_and_sig(sig_header) {
        Some(pair) => pair,
        None => return false,
    };

    // Reconstruct the signed payload PayMongo uses: "<timestamp>.<raw_body>"
    let mut signed_payload = timestamp.into_bytes();
    signed_payload.push(b'.');
    signed_payload.extend_from_slice(raw_body);

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };

    mac.update(&signed_payload);
    let computed = hex::encode(mac.finalize().into_bytes());

    // Constant-time string comparison
    computed == te_signature
}