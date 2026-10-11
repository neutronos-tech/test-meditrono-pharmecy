use contracts::catalog::{MedicineCatalogItem, MedicineChange};
use contracts::{
    auth::{PharmacyAccount, PharmacyLoginInput, PharmacyLoginResponse},
    sync::{SyncRequest, SyncResponse},
};
use serde::de::DeserializeOwned;
use std::{future::Future, time::Duration};
use uuid::Uuid;

#[derive(Clone)]
pub struct ServerClient {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RemoteError {
    #[error("server unavailable")]
    Unavailable,
    #[error("server rejected the session")]
    Unauthorized,
    #[error("invalid server response")]
    InvalidResponse,
}

#[derive(Debug, Clone)]
pub struct FullMedicineDownload {
    pub revision: u64,
    pub total: u64,
    pub medicines: Vec<MedicineCatalogItem>,
}

#[derive(Debug, Clone)]
pub struct MedicineChangesDownload {
    pub revision: u64,
    pub changes: Vec<MedicineChange>,
}

impl ServerClient {
    /// Client for an explicit base URL (tests point this at a local server).
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }

    pub fn new() -> Self {
        let base_url = std::env::var("PHARMACY_API_BASE_URL")
            .unwrap_or_else(|_| "https://pharmacy.neutorium.com".into());
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub async fn login(
        &self,
        username: String,
        password: String,
    ) -> Result<PharmacyLoginResponse, RemoteError> {
        self.client
            .post(self.url("/api/v1/auth/login"))
            .json(&PharmacyLoginInput { username, password })
            .send()
            .await
            .map_err(|_| RemoteError::Unavailable)?
            .error_for_status()
            .map_err(|error| {
                if error.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
                    RemoteError::Unauthorized
                } else {
                    RemoteError::Unavailable
                }
            })?
            .json()
            .await
            .map_err(|_| RemoteError::InvalidResponse)
    }

    pub async fn current_pharmacy(
        &self,
        session_token: &str,
    ) -> Result<PharmacyAccount, RemoteError> {
        self.client
            .get(self.url("/api/v1/auth/me"))
            .bearer_auth(session_token)
            .send()
            .await
            .map_err(|_| RemoteError::Unavailable)?
            .error_for_status()
            .map_err(|error| {
                if error.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
                    RemoteError::Unauthorized
                } else {
                    RemoteError::Unavailable
                }
            })?
            .json()
            .await
            .map_err(|_| RemoteError::InvalidResponse)
    }

    pub async fn logout(&self, session_token: &str) -> Result<(), RemoteError> {
        let response = self
            .client
            .post(self.url("/api/v1/auth/logout"))
            .bearer_auth(session_token)
            .send()
            .await
            .map_err(|_| RemoteError::Unavailable)?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(RemoteError::Unauthorized);
        }
        response
            .error_for_status()
            .map(|_| ())
            .map_err(|_| RemoteError::Unavailable)
    }

    /// Downloads the whole catalog page by page, keyed on the last id seen.
    pub async fn full_medicine_catalog(
        &self,
        session_token: &str,
    ) -> Result<FullMedicineDownload, RemoteError> {
        collect_pages(
            |after| self.medicine_page(session_token, after, CATALOG_PAGE_SIZE),
            CATALOG_PAGE_SIZE,
            Duration::from_millis(500),
        )
        .await
    }

    /// One page: (revision, total row count, items).
    async fn medicine_page(
        &self,
        session_token: &str,
        after: Option<Uuid>,
        limit: u32,
    ) -> Result<CatalogPage, RemoteError> {
        let mut query = vec![("limit", limit.to_string())];
        if let Some(after) = after {
            query.push(("after", after.to_string()));
        }
        let response = self
            .client
            .get(self.url("/api/v1/medicines"))
            .bearer_auth(session_token)
            .query(&query)
            .send()
            .await
            .map_err(|_| RemoteError::Unavailable)?
            .error_for_status()
            .map_err(|error| {
                if error.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
                    RemoteError::Unauthorized
                } else {
                    RemoteError::Unavailable
                }
            })?;
        let revision = header_number(&response, "x-medicine-revision")?;
        let total = header_number(&response, "x-medicine-count")?;
        let medicines = read_ndjson(response).await?;
        Ok((revision, total, medicines))
    }

    pub async fn medicine_changes(
        &self,
        session_token: &str,
        after: u64,
    ) -> Result<MedicineChangesDownload, RemoteError> {
        let response = self
            .client
            .get(self.url("/api/v1/medicines/changes"))
            .bearer_auth(session_token)
            .query(&[("after", after)])
            .send()
            .await
            .map_err(|_| RemoteError::Unavailable)?
            .error_for_status()
            .map_err(|error| {
                if error.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
                    RemoteError::Unauthorized
                } else {
                    RemoteError::Unavailable
                }
            })?;
        let revision = header_number(&response, "x-medicine-revision")?;
        let changes = read_ndjson(response).await?;
        Ok(MedicineChangesDownload { revision, changes })
    }

    pub async fn synchronize(
        &self,
        session_token: &str,
        request: SyncRequest,
    ) -> Result<SyncResponse, RemoteError> {
        self.client
            .post(self.url("/api/v1/sync"))
            .bearer_auth(session_token)
            .json(&request)
            .send()
            .await
            .map_err(|_| RemoteError::Unavailable)?
            .error_for_status()
            .map_err(|error| {
                if error.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
                    RemoteError::Unauthorized
                } else {
                    RemoteError::Unavailable
                }
            })?
            .json()
            .await
            .map_err(|_| RemoteError::InvalidResponse)
    }
}

impl Default for ServerClient {
    fn default() -> Self {
        Self::new()
    }
}

const CATALOG_PAGE_SIZE: u32 = 2000;
const CATALOG_ATTEMPTS: u32 = 4;

type CatalogPage = (u64, u64, Vec<MedicineCatalogItem>);

/// Walks every page of the catalog. The revision and total come from the first
/// page; a result shorter than `total` means the download was truncated.
async fn collect_pages<F, Fut>(
    mut fetch: F,
    page_size: u32,
    retry_delay: Duration,
) -> Result<FullMedicineDownload, RemoteError>
where
    F: FnMut(Option<Uuid>) -> Fut,
    Fut: Future<Output = Result<CatalogPage, RemoteError>>,
{
    let mut medicines = Vec::new();
    let mut after = None;
    let mut first = None;
    loop {
        let (revision, total, page) = fetch_with_retry(&mut fetch, after, retry_delay).await?;
        let (revision, total) = *first.get_or_insert((revision, total));
        let fetched = page.len();
        after = page.last().map(|item| item.id).or(after);
        medicines.extend(page);
        if fetched < page_size as usize {
            if (medicines.len() as u64) < total {
                return Err(RemoteError::InvalidResponse);
            }
            return Ok(FullMedicineDownload {
                revision,
                total,
                medicines,
            });
        }
    }
}

/// Retries a page only when the server was unreachable, with doubling delays.
async fn fetch_with_retry<F, Fut>(
    fetch: &mut F,
    after: Option<Uuid>,
    retry_delay: Duration,
) -> Result<CatalogPage, RemoteError>
where
    F: FnMut(Option<Uuid>) -> Fut,
    Fut: Future<Output = Result<CatalogPage, RemoteError>>,
{
    let mut attempt = 0;
    loop {
        match fetch(after).await {
            Err(RemoteError::Unavailable) if attempt + 1 < CATALOG_ATTEMPTS => {
                tokio::time::sleep(retry_delay * 2u32.pow(attempt)).await;
                attempt += 1;
            }
            result => return result,
        }
    }
}

fn header_number(response: &reqwest::Response, name: &str) -> Result<u64, RemoteError> {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok())
        .ok_or(RemoteError::InvalidResponse)
}

fn parse_line<T: DeserializeOwned>(line: &[u8]) -> Result<Option<T>, RemoteError> {
    if line.iter().all(u8::is_ascii_whitespace) {
        return Ok(None);
    }
    serde_json::from_slice(line)
        .map(Some)
        .map_err(|_| RemoteError::InvalidResponse)
}

async fn read_ndjson<T: DeserializeOwned>(
    mut response: reqwest::Response,
) -> Result<Vec<T>, RemoteError> {
    let mut values = Vec::new();
    let mut pending = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| RemoteError::Unavailable)?
    {
        pending.extend_from_slice(&chunk);
        while let Some(end) = pending.iter().position(|byte| *byte == b'\n') {
            let line: Vec<u8> = pending.drain(..=end).collect();
            if let Some(value) = parse_line(&line)? {
                values.push(value);
            }
        }
    }
    if let Some(value) = parse_line(&pending)? {
        values.push(value);
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_ndjson_fails_closed() {
        assert!(parse_line::<MedicineCatalogItem>(b"not json\n").is_err());
        assert!(parse_line::<MedicineCatalogItem>(b" \n").unwrap().is_none());
    }

    fn items(count: u128) -> Vec<MedicineCatalogItem> {
        (1..=count)
            .map(|index| MedicineCatalogItem {
                id: Uuid::from_u128(index),
                name: format!("Medicine {index}"),
                generic_name: None,
                strength: None,
                dosage_form: None,
                manufacturer: None,
                package_description: None,
                reference_price: None,
                active: true,
            })
            .collect()
    }

    /// What the server returns for `GET /api/v1/medicines?after=&limit=`.
    fn serve(
        all: &[MedicineCatalogItem],
        after: Option<Uuid>,
        limit: usize,
    ) -> Vec<MedicineCatalogItem> {
        all.iter()
            .filter(|item| after.is_none_or(|after| item.id > after))
            .take(limit)
            .cloned()
            .collect()
    }

    #[tokio::test]
    async fn pages_collect_every_row_in_order() {
        let all = items(5);
        let total = all.len() as u64;
        let calls = std::cell::Cell::new(0);
        let download = collect_pages(
            |after| {
                calls.set(calls.get() + 1);
                let page = serve(&all, after, 2);
                async move { Ok((7, total, page)) }
            },
            2,
            Duration::ZERO,
        )
        .await
        .unwrap();
        assert_eq!(download.revision, 7);
        assert_eq!(download.total, 5);
        assert_eq!(download.medicines.len(), 5);
        assert!(
            download
                .medicines
                .iter()
                .zip(&all)
                .all(|(got, want)| got.id == want.id)
        );
        assert_eq!(calls.get(), 3);
    }

    #[tokio::test]
    async fn exact_multiple_of_page_size_stops_on_empty_page() {
        let all = items(4);
        let total = all.len() as u64;
        let calls = std::cell::Cell::new(0);
        let download = collect_pages(
            |after| {
                calls.set(calls.get() + 1);
                let page = serve(&all, after, 2);
                async move { Ok((1, total, page)) }
            },
            2,
            Duration::ZERO,
        )
        .await
        .unwrap();
        assert_eq!(download.medicines.len(), 4);
        assert_eq!(calls.get(), 3);
    }

    #[tokio::test]
    async fn empty_catalog_is_one_request() {
        let download = collect_pages(|_| async { Ok((0, 0, Vec::new())) }, 2, Duration::ZERO)
            .await
            .unwrap();
        assert!(download.medicines.is_empty());
        assert_eq!(download.revision, 0);
    }

    #[tokio::test]
    async fn transient_unavailable_pages_are_retried() {
        let all = items(3);
        let total = all.len() as u64;
        let calls = std::cell::Cell::new(0);
        let download = collect_pages(
            |after| {
                calls.set(calls.get() + 1);
                let attempt = calls.get();
                let page = serve(&all, after, 2);
                async move {
                    if attempt <= 2 {
                        Err(RemoteError::Unavailable)
                    } else {
                        Ok((1, total, page))
                    }
                }
            },
            2,
            Duration::ZERO,
        )
        .await
        .unwrap();
        assert_eq!(download.medicines.len(), 3);
        assert_eq!(calls.get(), 4);
    }

    #[tokio::test]
    async fn gives_up_after_the_last_attempt() {
        let calls = std::cell::Cell::new(0);
        let result = collect_pages(
            |_| {
                calls.set(calls.get() + 1);
                async { Err(RemoteError::Unavailable) }
            },
            2,
            Duration::ZERO,
        )
        .await;
        assert_eq!(result.unwrap_err(), RemoteError::Unavailable);
        assert_eq!(calls.get(), CATALOG_ATTEMPTS);
    }

    #[tokio::test]
    async fn unauthorized_and_invalid_responses_are_not_retried() {
        for error in [RemoteError::Unauthorized, RemoteError::InvalidResponse] {
            let calls = std::cell::Cell::new(0);
            let result = collect_pages(
                |_| {
                    calls.set(calls.get() + 1);
                    async move { Err(error) }
                },
                2,
                Duration::ZERO,
            )
            .await;
            assert_eq!(result.unwrap_err(), error);
            assert_eq!(calls.get(), 1);
        }
    }

    #[tokio::test]
    async fn truncated_download_is_rejected() {
        let all = items(3);
        let result = collect_pages(
            |after| {
                let page = serve(&all, after, 2);
                async move { Ok((1, 4, page)) }
            },
            2,
            Duration::ZERO,
        )
        .await;
        assert_eq!(result.unwrap_err(), RemoteError::InvalidResponse);
    }

    #[tokio::test]
    async fn revision_comes_from_the_first_page() {
        let all = items(3);
        let calls = std::cell::Cell::new(0);
        let download = collect_pages(
            |after| {
                calls.set(calls.get() + 1);
                let revision = 10 + calls.get();
                let page = serve(&all, after, 2);
                async move { Ok((revision, 3, page)) }
            },
            2,
            Duration::ZERO,
        )
        .await
        .unwrap();
        assert_eq!(download.revision, 11);
    }
}
