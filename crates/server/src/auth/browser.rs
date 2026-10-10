use crate::{
    app::{AppState, LoginError, SessionKind},
    auth::middleware::BrowserCredential,
    error::AppError,
};
use askama::Template;
use axum::{
    Form,
    extract::{Extension, State},
    http::{HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
};
use contracts::auth::PharmacyAccount;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage<'a> {
    error: &'a str,
}

#[derive(Template)]
#[template(path = "index.html")]
struct DashboardPage<'a> {
    username: &'a str,
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

pub async fn login_page() -> Result<Html<String>, AppError> {
    Ok(Html(LoginPage { error: "" }.render()?))
}

pub async fn login(
    State(app): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Result<Response, AppError> {
    let auth = match app
        .login_pharmacy(&form.username, &form.password, SessionKind::Browser)
        .await
    {
        Ok(auth) => auth,
        Err(LoginError::InvalidCredentials) => {
            return Ok(Html(
                LoginPage {
                    error: "Invalid username or password.",
                }
                .render()?,
            )
            .into_response());
        }
        Err(LoginError::Database) => return Err(AppError::Internal),
    };
    let Some(csrf) = auth.csrf_token else {
        return Err(AppError::Internal);
    };
    let secure = secure_cookies();
    let secure_attr = if secure { "; Secure" } else { "" };
    let session_cookie = format!(
        "pharmacy_session={}; HttpOnly; SameSite=Lax; Path=/; Max-Age=2592000{}",
        auth.session_token, secure_attr
    );
    let csrf_cookie = format!(
        "pharmacy_csrf={}; SameSite=Lax; Path=/; Max-Age=2592000{}",
        csrf, secure_attr
    );
    let mut response = Redirect::to("/").into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&session_cookie).map_err(|_| AppError::Internal)?,
    );
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&csrf_cookie).map_err(|_| AppError::Internal)?,
    );
    Ok(response)
}

pub async fn dashboard(
    Extension(pharmacy): Extension<PharmacyAccount>,
) -> Result<Html<String>, AppError> {
    Ok(Html(
        DashboardPage {
            username: &pharmacy.username,
        }
        .render()?,
    ))
}

pub async fn logout(
    State(app): State<AppState>,
    Extension(credential): Extension<BrowserCredential>,
) -> Result<Response, AppError> {
    app.logout_session(&credential.token)
        .await
        .map_err(|_| AppError::Unauthorized)?;
    let secure_attr = if secure_cookies() { "; Secure" } else { "" };
    let mut response = (StatusCode::SEE_OTHER, [(header::LOCATION, "/login")]).into_response();
    for cookie in ["pharmacy_session", "pharmacy_csrf"] {
        let clear = format!("{cookie}=; Path=/; Max-Age=0; SameSite=Lax{secure_attr}");
        response.headers_mut().append(
            header::SET_COOKIE,
            HeaderValue::from_str(&clear).map_err(|_| AppError::Internal)?,
        );
    }
    Ok(response)
}

pub fn secure_cookies() -> bool {
    !matches!(
        std::env::var("PHARMACY_SECURE_COOKIES").as_deref(),
        Ok("false") | Ok("0")
    )
}
