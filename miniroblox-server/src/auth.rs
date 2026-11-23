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

// game: loopback redirect + game client id + game secret
pub fn login_with_code(code: &str) -> Option<String> {
    let secret = std::env::var("GOOGLE_CLIENT_SECRET").ok()?;
    exchange_and_verify(code, GAME_CLIENT_ID, &secret, GAME_REDIRECT)
}

// admin web: its own client id + secret + https redirect
pub fn login_with_code_web(code: &str, redirect_uri: &str) -> Option<String> {
    let secret = std::env::var("WEB_CLIENT_SECRET").ok()?;
    exchange_and_verify(code, WEB_CLIENT_ID, &secret, redirect_uri)
}

fn exchange_and_verify(
    code: &str,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
) -> Option<String> {
    // Dev mode (DEV=1): "dev:name" bypasses Google — for local testing only.
    if std::env::var("DEV").map(|v| v == "1").unwrap_or(false) {
        if let Some(name) = code.strip_prefix("dev:") {
            return Some(name.to_string());
        }
    }
    let resp = match ureq::post("https://oauth2.googleapis.com/token")
        .send_form(&[
            ("code", code),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
        ]) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[auth] token POST network error: {}", e);
            return None;
        }
    };
    eprintln!("[auth] token exchange status: {}", resp.status());
    if resp.status() >= 400 {
        let status = resp.status();
        let body = resp.into_string().unwrap_or_default();
        eprintln!("[auth] token exchange failed ({}): {}", status, body);
        return None;
    }
    let info: TokenResponse = match resp.into_json() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("[auth] token json parse error: {}", e);
            return None;
        }
    };
    let id_token = match info.id_token {
        Some(t) => t,
        None => {
            eprintln!("[auth] no id_token in token response");
            return None;
        }
    };
    verify_google_token(&id_token, client_id)
}

// Verify an ID token: ask Google's tokeninfo endpoint, check audience + email.
fn verify_google_token(id_token: &str, expected_aud: &str) -> Option<String> {
    let url = format!(
        "https://oauth2.googleapis.com/tokeninfo?id_token={}",
        id_token
    );
    let resp = match ureq::get(&url).call() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[auth] tokeninfo network error: {}", e);
            return None;
        }
    };
    let info: TokenInfo = match resp.into_json() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("[auth] tokeninfo json error: {}", e);
            return None;
        }
    };
    let email = match info.email {
        Some(e) => e,
        None => {
            eprintln!("[auth] tokeninfo has no email");
            return None;
        }
    };
    let aud = match info.aud {
        Some(a) => a,
        None => {
            eprintln!("[auth] tokeninfo has no aud");
            return None;
        }
    };
    // Reject tokens issued for a different client (anti-spoofing).
    if aud != expected_aud {
        eprintln!("[auth] tokeninfo aud mismatch: {} != {}", aud, expected_aud);
        return None;
    }
    Some(email)
}
