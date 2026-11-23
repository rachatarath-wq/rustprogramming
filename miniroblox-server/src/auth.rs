// Google OAuth: exchange the authorization code for an ID token, then verify it.
use serde::Deserialize;

// Public client id — this is what the game uses to request the token.
const GAME_CLIENT_ID: &str =
    "682624579295-hc9fqes8btvs795f2udl0mivhelc349i.apps.googleusercontent.com";
const WEB_CLIENT_ID: &str =
    "682624579295-rtgjcfqmkk0deda7ji43r4lcje5jsiqp.apps.googleusercontent.com";

const GAME_REDIRECT: &str = "http://127.0.0.1:8080";

#[derive(Deserialize)]
struct TokenResponse {
    id_token: Option<String>,
}

#[derive(Deserialize)]
struct TokenInfo {
    email: Option<String>,
    aud: Option<String>,
}

// game: loopback redirect + game client id
pub fn login_with_code(code: &str) -> Option<String> {
    exchange_and_verify(code, GAME_CLIENT_ID, GAME_REDIRECT)
}

// admin web: its own client id + https redirect
pub fn login_with_code_web(code: &str, redirect_uri: &str) -> Option<String> {
    exchange_and_verify(code, WEB_CLIENT_ID, redirect_uri)
}

fn exchange_and_verify(code: &str, client_id: &str, redirect_uri: &str) -> Option<String> {
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
            ("client_id", client_id),
            ("client_secret", secret.as_str()),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .ok()?;
    let info: TokenResponse = resp.into_json().ok()?;
    let id_token = info.id_token?;
    verify_google_token(&id_token, client_id)
}

// Verify an ID token: ask Google's tokeninfo endpoint, check audience + email.
fn verify_google_token(id_token: &str, expected_aud: &str) -> Option<String> {
    let url = format!(
        "https://oauth2.googleapis.com/tokeninfo?id_token={}",
        id_token
    );
    let resp = ureq::get(&url).call().ok()?;
    let info: TokenInfo = resp.into_json().ok()?;
    let email = info.email?;
    let aud = info.aud?;
    // Reject tokens issued for a different client (anti-spoofing).
    if aud != expected_aud {
        return None;
    }
    Some(email)
}
