use crate::{Backend, Error, RemoteError, SavedRemoteAuth, ServerClient};
use contracts::sync::SyncSummary;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("local database operation failed")]
    Database(#[from] Error),
    #[error("remote synchronization failed")]
    Remote(#[from] RemoteError),
}

pub async fn sync_with_server(
    database_path: PathBuf,
    client: ServerClient,
    auth: SavedRemoteAuth,
) -> Result<SyncSummary, SyncError> {
    let request = {
        let backend = Backend::open(&database_path)?;
        backend.prepare_sync_request()?
    };
    let response = client.synchronize(&auth.session_token, request).await?;
    let mut backend = Backend::open(database_path)?;
    Ok(backend.apply_sync_response(response)?)
}
