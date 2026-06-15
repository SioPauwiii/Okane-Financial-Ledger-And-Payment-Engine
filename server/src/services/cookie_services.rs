use axum_extra::extract::cookie::{Cookie, SameSite};

pub const AUTH_COOKIE_NAME: &str = "access_token";

// pub async fn read_auth_cookie(cookie: Option<Cookie<'_>>) -> Option<String> {
//     cookie
//         .filter(|c| c.name() == AUTH_COOKIE_NAME)
//         .map(|c| c.value().to_string())
// }

pub fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into())
}

pub fn build_auth_cookie(token: &str) -> Cookie<'static> {
    let mut cookie = Cookie::new(AUTH_COOKIE_NAME, token.to_owned());
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::None);
    cookie.set_path("/");

    if std::env::var("COOKIE_SECURE")
        .unwrap_or_else(|_| "false".into())
        .eq_ignore_ascii_case("true")
    {
        cookie.set_secure(true);
    }

    cookie
}

pub fn clear_auth_cookie() -> Cookie<'static> {
    let mut cookie = Cookie::new(AUTH_COOKIE_NAME, "");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::None);
    cookie.set_path("/");
    cookie.make_removal();
    cookie
}