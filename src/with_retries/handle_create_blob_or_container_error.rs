use my_azure_storage_sdk::AzureStorageError;

pub async fn handle_create_blob_or_container_error(
    err: AzureStorageError,
    attempt_no: usize,
    max_attempts: usize,
) -> Result<(), AzureStorageError> {
    if attempt_no >= max_attempts {
        return Err(err);
    }

    match err {
        AzureStorageError::ContainerNotFound => {
            return Err(err);
        }
        AzureStorageError::BlobNotFound => {
            return Err(err);
        }
        AzureStorageError::BlobAlreadyExists => {
            return Err(err);
        }
        AzureStorageError::ContainerBeingDeleted => {
            return Ok(());
        }
        AzureStorageError::ContainerAlreadyExists => {
            return Err(err);
        }
        AzureStorageError::InvalidPageRange => {
            return Err(err);
        }
        AzureStorageError::RequestBodyTooLarge => {
            return Err(err);
        }
        AzureStorageError::InvalidResourceName => {
            return Err(err);
        }
        AzureStorageError::IoError(_) => {}
        AzureStorageError::HyperError(_) => {}
        AzureStorageError::Timeout => {}
        AzureStorageError::UnknownError { msg: _ } => {}
    }

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    Ok(())
}
