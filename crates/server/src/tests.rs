//! Server tests against a real Postgres database (see `test_support`).

use crate::{app::AppState, route, test_support::fresh_db};
use axum::{
    body::{Body, to_bytes},
    http::{Request, Response, StatusCode, header},
};
use contracts::{
    auth::{PharmacyAccount, PharmacyLoginInput, PharmacyLoginResponse},
    catalog::{MedicineCatalogItem, MedicineChange},
};
use serde::de::DeserializeOwned;
use sqlx_core::raw_sql::raw_sql;
use tower::ServiceExt;
use uuid::Uuid;

const USERNAME: &str = "pharmacy-one";
const PASSWORD: &str = "correct horse";

async fn send(app: &AppState, request: Request<Body>) -> Response<Body> {
    route::router(app.clone()).oneshot(request).await.unwrap()
}

fn get(uri: &str, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method("GET").uri(uri);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    builder.body(Body::empty()).unwrap()
}

async fn body_bytes(response: Response<Body>) -> Vec<u8> {
    to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec()
}

fn ndjson<T: DeserializeOwned>(bytes: &[u8]) -> Vec<T> {
    bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.iter().all(u8::is_ascii_whitespace))
        .map(|line| serde_json::from_slice(line).unwrap())
        .collect()
}

async fn sign_in(app: &AppState) -> String {
    app.bootstrap_pharmacy(USERNAME, PASSWORD).await.unwrap();
    let body = serde_json::to_vec(&PharmacyLoginInput {
        username: USERNAME.into(),
        password: PASSWORD.into(),
    })
    .unwrap();
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap();
    let response = send(app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    serde_json::from_slice::<PharmacyLoginResponse>(&body_bytes(response).await)
        .unwrap()
        .session_token
}

async fn seed_catalog(app: &AppState, count: u32) {
    raw_sql(&format!(
        "INSERT INTO medicine_catalog (id, brand_name) \
         SELECT gen_random_uuid(), 'Brand ' || lpad(g::text, 4, '0') \
         FROM generate_series(1, {count}) AS g"
    ))
    .execute(&app.pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn protected_routes_require_a_session() {
    let db = fresh_db().await;
    for uri in [
        "/api/v1/medicines",
        "/api/v1/medicines/changes?after=0",
        "/api/v1/auth/me",
    ] {
        let missing = send(&db.app, get(uri, None)).await;
        assert_eq!(
            missing.status(),
            StatusCode::UNAUTHORIZED,
            "{uri} without a token"
        );
        let wrong = send(&db.app, get(uri, Some("not-a-session"))).await;
        assert_eq!(
            wrong.status(),
            StatusCode::UNAUTHORIZED,
            "{uri} with a bad token"
        );
    }
    assert_eq!(
        send(&db.app, get("/health", None)).await.status(),
        StatusCode::OK
    );
    db.drop_db().await;
}

#[tokio::test]
async fn login_token_works_until_logout_and_wrong_password_is_refused() {
    let db = fresh_db().await;
    let token = sign_in(&db.app).await;

    let me = send(&db.app, get("/api/v1/auth/me", Some(&token))).await;
    assert_eq!(me.status(), StatusCode::OK);
    let account: PharmacyAccount = serde_json::from_slice(&body_bytes(me).await).unwrap();
    assert_eq!(account.username, USERNAME);

    let wrong = serde_json::to_vec(&PharmacyLoginInput {
        username: USERNAME.into(),
        password: "wrong".into(),
    })
    .unwrap();
    let refused = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(wrong))
        .unwrap();
    assert_eq!(
        send(&db.app, refused).await.status(),
        StatusCode::UNAUTHORIZED
    );

    let logout = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/logout")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(&db.app, logout).await.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        send(&db.app, get("/api/v1/auth/me", Some(&token)))
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    db.drop_db().await;
}

#[tokio::test]
async fn catalog_pages_cover_every_row_once_in_id_order() {
    let db = fresh_db().await;
    let token = sign_in(&db.app).await;
    seed_catalog(&db.app, 5100).await;

    let mut after: Option<Uuid> = None;
    let mut seen: Vec<Uuid> = Vec::new();
    let mut page_sizes = Vec::new();
    loop {
        let uri = match after {
            Some(id) => format!("/api/v1/medicines?after={id}"),
            None => "/api/v1/medicines".to_string(),
        };
        let response = send(&db.app, get(&uri, Some(&token))).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()["x-medicine-count"].to_str().unwrap(),
            "5100"
        );
        assert_eq!(
            response.headers()["x-medicine-revision"].to_str().unwrap(),
            "0"
        );
        let page: Vec<MedicineCatalogItem> = ndjson(&body_bytes(response).await);
        page_sizes.push(page.len());
        if page.is_empty() {
            break;
        }
        after = page.last().map(|item| item.id);
        seen.extend(page.into_iter().map(|item| item.id));
    }

    assert_eq!(
        page_sizes,
        [2000, 2000, 1100, 0],
        "default page size is 2000"
    );
    assert_eq!(seen.len(), 5100);
    assert!(
        seen.windows(2).all(|pair| pair[0] < pair[1]),
        "ids strictly ascending"
    );
    db.drop_db().await;
}

#[tokio::test]
async fn catalog_limit_is_clamped_to_one_and_five_thousand() {
    let db = fresh_db().await;
    let token = sign_in(&db.app).await;
    seed_catalog(&db.app, 5100).await;

    let zero: Vec<MedicineCatalogItem> = ndjson(
        &body_bytes(send(&db.app, get("/api/v1/medicines?limit=0", Some(&token))).await).await,
    );
    assert_eq!(zero.len(), 1);

    let huge: Vec<MedicineCatalogItem> = ndjson(
        &body_bytes(send(&db.app, get("/api/v1/medicines?limit=100000", Some(&token))).await).await,
    );
    assert_eq!(huge.len(), 5000);
    db.drop_db().await;
}

#[tokio::test]
async fn changes_return_only_newer_revisions_in_order() {
    let db = fresh_db().await;
    let token = sign_in(&db.app).await;
    seed_catalog(&db.app, 3).await;
    raw_sql(
        "INSERT INTO medicine_catalog_changes (medicine_id) \
         SELECT id FROM medicine_catalog ORDER BY brand_name",
    )
    .execute(&db.app.pool)
    .await
    .unwrap();

    let all = send(
        &db.app,
        get("/api/v1/medicines/changes?after=0", Some(&token)),
    )
    .await;
    assert_eq!(all.headers()["x-medicine-revision"].to_str().unwrap(), "3");
    let changes: Vec<MedicineChange> = ndjson(&body_bytes(all).await);
    let revisions: Vec<u64> = changes.iter().map(|change| change.revision).collect();
    assert_eq!(revisions, [1, 2, 3]);

    let newer = send(
        &db.app,
        get("/api/v1/medicines/changes?after=2", Some(&token)),
    )
    .await;
    let changes: Vec<MedicineChange> = ndjson(&body_bytes(newer).await);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].revision, 3);
    db.drop_db().await;
}

#[tokio::test]
async fn malformed_cursor_is_a_client_error() {
    let db = fresh_db().await;
    let token = sign_in(&db.app).await;
    let response = send(
        &db.app,
        get("/api/v1/medicines?after=not-a-uuid", Some(&token)),
    )
    .await;
    assert!(
        response.status().is_client_error(),
        "got {}",
        response.status()
    );
    db.drop_db().await;
}

#[tokio::test]
async fn client_downloads_whole_catalog_over_real_http() {
    let db = fresh_db().await;
    db.app.bootstrap_pharmacy(USERNAME, PASSWORD).await.unwrap();
    seed_catalog(&db.app, 5100).await;
    raw_sql(
        "INSERT INTO medicine_catalog_changes (medicine_id) \
         SELECT id FROM medicine_catalog ORDER BY brand_name LIMIT 2",
    )
    .execute(&db.app.pool)
    .await
    .unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let router = route::router(db.app.clone());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

    let client = backend::ServerClient::with_base_url(format!("http://{address}"));
    let login = client
        .login(USERNAME.into(), PASSWORD.into())
        .await
        .expect("login over HTTP");
    let download = client
        .full_medicine_catalog(&login.session_token)
        .await
        .expect("paged download over HTTP");
    assert_eq!(download.total, 5100);
    assert_eq!(download.medicines.len(), 5100);
    assert_eq!(download.revision, 2);

    let dir = tempfile::tempdir().unwrap();
    let mut local = backend::Backend::open(dir.path().join("pharmacy.db")).unwrap();
    let revision = download.revision;
    local
        .import_medicine_catalog(download.medicines, revision)
        .unwrap();
    assert_eq!(local.search_medicines("Brand 0042", 10).unwrap().len(), 1);

    let changes = client
        .medicine_changes(&login.session_token, revision)
        .await
        .expect("changes over HTTP");
    assert!(changes.changes.is_empty());
    db.drop_db().await;
}

#[cfg(debug_assertions)]
#[tokio::test]
async fn developer_token_authenticates_only_when_enabled() {
    let mut db = fresh_db().await;
    let token = "0123456789abcdef-dev";
    assert_eq!(
        send(&db.app, get("/api/v1/auth/me", Some(token)))
            .await
            .status(),
        StatusCode::UNAUTHORIZED,
        "not accepted before it is enabled"
    );
    assert!(db.app.enable_dev_token("too-short").await.is_err());

    db.app.enable_dev_token(token).await.unwrap();
    let me = send(&db.app, get("/api/v1/auth/me", Some(token))).await;
    assert_eq!(me.status(), StatusCode::OK);
    let account: PharmacyAccount = serde_json::from_slice(&body_bytes(me).await).unwrap();
    assert_eq!(account.username, contracts::auth::dev::USERNAME);
    assert_eq!(
        send(&db.app, get("/api/v1/medicines", Some(token)))
            .await
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        send(
            &db.app,
            get("/api/v1/auth/me", Some("0123456789abcdef-other"))
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
    db.drop_db().await;
}

fn renew(token: &str, fingerprint: &str) -> Request<Body> {
    let body = serde_json::to_vec(&contracts::licence::LicenceRenewInput {
        device_fingerprint: fingerprint.into(),
        device_name: "Counter PC".into(),
    })
    .unwrap();
    Request::builder()
        .method("POST")
        .uri("/api/v1/licence/renew")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap()
}

#[tokio::test]
async fn first_device_gets_a_trial_licence_and_second_device_must_pay() {
    use contracts::licence::{LicenceState, SignedLicence, TrustedKey, evaluate, verify};
    let db = fresh_db().await;
    let token = sign_in(&db.app).await;
    let trusted = [TrustedKey {
        key_id: 1,
        public_key: db.app.licence_key.as_ref().unwrap().public_key(),
    }];

    let response = send(&db.app, renew(&token, "device-a")).await;
    assert_eq!(response.status(), StatusCode::OK);
    let licence: SignedLicence = serde_json::from_slice(&body_bytes(response).await).unwrap();
    let payload = verify(&licence, &trusted, "device-a").unwrap();
    let now = time::OffsetDateTime::now_utc();
    assert!(matches!(
        evaluate(&payload, now, now),
        Ok(LicenceState::Active { days_left: 29 | 30 })
    ));

    let response = send(&db.app, renew(&token, "device-b")).await;
    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);

    let device_b: i64 = sqlx_core::query_scalar::query_scalar(
        "SELECT id FROM pharmacy_devices WHERE name='Counter PC' ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&db.app.pool)
    .await
    .unwrap();
    db.app.extend_device(device_b, 30).await.unwrap();
    let response = send(&db.app, renew(&token, "device-b")).await;
    assert_eq!(response.status(), StatusCode::OK);

    let audits: i64 = sqlx_core::query_scalar::query_scalar("SELECT COUNT(*) FROM audit_log")
        .fetch_one(&db.app.pool)
        .await
        .unwrap();
    assert_eq!(audits, 3);
    db.drop_db().await;
}

fn signup(username: &str) -> Request<Body> {
    let body = serde_json::to_vec(&contracts::saas::SignupInput {
        username: username.into(),
        password: "long enough".into(),
    })
    .unwrap();
    Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap()
}

#[tokio::test]
async fn signup_creates_an_account_and_lists_plans_and_devices() {
    let db = fresh_db().await;
    let plans = send(&db.app, get("/api/v1/plans", None)).await;
    assert_eq!(plans.status(), StatusCode::OK);
    let plans: Vec<contracts::saas::Plan> =
        serde_json::from_slice(&body_bytes(plans).await).unwrap();
    assert_eq!(plans[0].code, "standard");

    let response = send(&db.app, signup("Rahman-Pharmacy")).await;
    assert_eq!(response.status(), StatusCode::OK);
    let login: PharmacyLoginResponse = serde_json::from_slice(&body_bytes(response).await).unwrap();
    assert_eq!(login.pharmacy.username, "rahman-pharmacy");
    let again = send(&db.app, signup("rahman-pharmacy")).await;
    assert_eq!(again.status(), StatusCode::CONFLICT);
    let bad = send(&db.app, signup("a b")).await;
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);

    let renewed = send(&db.app, renew(&login.session_token, "pc-1")).await;
    assert_eq!(renewed.status(), StatusCode::OK);
    let devices = send(&db.app, get("/api/v1/devices", Some(&login.session_token))).await;
    let devices: Vec<contracts::saas::Device> =
        serde_json::from_slice(&body_bytes(devices).await).unwrap();
    assert_eq!(devices.len(), 1);
    db.drop_db().await;
}
