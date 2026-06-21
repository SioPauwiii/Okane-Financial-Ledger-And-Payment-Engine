use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub http_client:reqwest::Client,
    pub paymongo_secret_key:String,
    pub paymongo_webhook_secret:String,    
}