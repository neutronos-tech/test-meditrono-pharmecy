//! Live tests against a running pharmacy server.
//!
//! They need `PHARMACY_TEST_USERNAME` and `PHARMACY_TEST_PASSWORD`, and use
//! `PHARMACY_API_BASE_URL` (default `https://pharmacy.neutorium.com`). When the
//! server is unreachable or no account is configured, each test prints a
//! SKIPPED line and passes.

use backend::{Backend, RemoteError, ServerClient};
use std::{collections::HashSet, time::Duration};

struct Live {
    client: ServerClient,
    token: String,
}

async fn connect() -> Option<Live> {
    let base = std::env::var("PHARMACY_API_BASE_URL")
        .unwrap_or_else(|_| "https://pharmacy.neutorium.com".into());
    let health = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap()
        .get(format!("{}/health", base.trim_end_matches('/')))
        .send()
        .await;
    if !health.is_ok_and(|response| response.status().is_success()) {
        eprintln!("SKIPPED: server not reachable at {base}");
        return None;
    }
    let client = ServerClient::with_base_url(base);
    // A developer token (debug server only) skips the password login.
    if let Ok(token) = std::env::var("PHARMACY_TEST_TOKEN") {
        return Some(Live { client, token });
    }
    let (Ok(username), Ok(password)) = (
        std::env::var("PHARMACY_TEST_USERNAME"),
        std::env::var("PHARMACY_TEST_PASSWORD"),
    ) else {
        eprintln!("SKIPPED: PHARMACY_TEST_TOKEN or PHARMACY_TEST_USERNAME/PASSWORD not set");
        return None;
    };
    let login = client
        .login(username, password)
        .await
        .expect("the configured test account must be able to sign in");
    Some(Live {
        client,
        token: login.session_token,
    })
}

#[tokio::test]
async fn bad_token_is_rejected() {
    let Some(live) = connect().await else { return };
    let result = live.client.full_medicine_catalog("not-a-real-token").await;
    assert_eq!(result.err(), Some(RemoteError::Unauthorized));
}

#[tokio::test]
async fn full_download_matches_server_total_and_imports_locally() {
    let Some(live) = connect().await else { return };
    let download = live
        .client
        .full_medicine_catalog(&live.token)
        .await
        .expect("full catalog download");
    assert!(download.total > 0, "server catalog is empty");
    assert_eq!(download.medicines.len() as u64, download.total);
    let unique: HashSet<_> = download.medicines.iter().map(|item| item.id).collect();
    assert_eq!(unique.len(), download.medicines.len(), "ids must be unique");

    let dir = tempfile::tempdir().unwrap();
    let mut local = Backend::open(dir.path().join("pharmacy.db")).unwrap();
    let revision = download.revision;
    local
        .import_medicine_catalog(download.medicines, revision)
        .unwrap();
    assert_eq!(local.medicine_revision().unwrap(), revision);
    assert!(!local.search_medicines("", u32::MAX).unwrap().is_empty());
}

#[tokio::test]
async fn changes_after_current_revision_are_empty() {
    let Some(live) = connect().await else { return };
    let download = live
        .client
        .full_medicine_catalog(&live.token)
        .await
        .expect("full catalog download");
    let changes = live
        .client
        .medicine_changes(&live.token, download.revision)
        .await
        .expect("changes request");
    assert!(changes.changes.is_empty());
    assert_eq!(changes.revision, download.revision);
}
