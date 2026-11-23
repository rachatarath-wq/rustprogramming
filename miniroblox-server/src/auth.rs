// Google OAuth: exchange the authorization code for an ID token, then verify it.
use serde::Deserialize;

// Public client id — this is what the game uses to request the token.
const GOOGLE_CLIENT_ID: &str =
    "682624579295-hc9fqes8btvs795f2udl0mivhelc349i.apps.googleusercontent.com";

const REDIRECT_URI: &str = "http://127.0.0.1:8080";

#[derive(Deserialize)]
struct TokenResponse {
    id_token: Option<String>,
}

#[derive(Deserialize)]
struct TokenInfo {
    email: Option<String>,
    aud: Option<String>,
}

// Full login: exchange code -> id_token -> verify -> email.
// (game uses the loopback redirect; the admin web passes its own redirect)
pub fn login_with_code(code: &str) -> Option<String> {
    login_with_code_redirect(code, REDIRECT_URI)
}

pub fn login_with_code_redirect(code: &str, redirect_uri: &str) -> Option<String> {
    // Dev mode (DEV=1): "dev:name" bypasses Google — for local testing only.
    if std::env::var("DEV").map(|v| v == "1").unwrap_or(false) {
        if let Some(name) = code.strip_prefix("dev:") {
            return Some(name.to_string());
        }
    }
    // The client secret stays on the server (never in the game binary).
    let secret = std::env::var("GOOGLE_CLIENT_SECRET").ok()?;
    let resp = ureq::post("https://oauth2.googleapis.com/token")
        .send_form(&[
            ("code", code),
            ("client_id", GOOGLE_CLIENT_ID),
            ("client_secret", secret.as_str()),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .ok()?;
    let info: TokenResponse = resp.into_json().ok()?;
    let id_token = info.id_token?;
    verify_google_token(&id_token)
}

// Verify an ID token: ask Google's tokeninfo endpoint, check audience + email.
fn verify_google_token(id_token: &str) -> Option<String> {
    let url = format!(
        "https://oauth2.googleapis.com/tokeninfo?id_token={}",
        id_token
    );
    let resp = ureq::get(&url).call().ok()?;
    let info: TokenInfo = resp.into_json().ok()?;
    let email = info.email?;
    let aud = info.aud?;
    // Reject tokens issued for a different client (anti-spoofing).
    if aud != GOOGLE_CLIENT_ID {
        return None;
    }
    Some(email)
}
