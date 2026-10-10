use contracts::catalog::{MedicineCatalogItem, MedicineChange};
use contracts::{
    auth::{PharmacyAccount, PharmacyLoginInput, PharmacyLoginResponse},
    sync::{SyncRequest, SyncResponse},
};
use serde::de::DeserializeOwned;

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

    pub async fn full_medicine_catalog(
        &self,
        session_token: &str,
    ) -> Result<FullMedicineDownload, RemoteError> {
        let response = self
            .client
            .get(self.url("/api/v1/medicines"))
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
            })?;
        let revision = header_number(&response, "x-medicine-revision")?;
        let total = header_number(&response, "x-medicine-count")?;
        let medicines = read_ndjson(response).await?;
        Ok(FullMedicineDownload {
            revision,
            total,
            medicines,
        })
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
}
