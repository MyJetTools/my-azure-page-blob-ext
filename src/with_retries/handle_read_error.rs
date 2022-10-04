use my_azure_storage_sdk::{page_blob::AzurePageBlobStorage, AzureStorageError};

pub async fn handle_read_error(
    page_blob: &AzurePageBlobStorage,
    err: AzureStorageError,
    create_container_if_not_exist: bool,
    create_blob_if_not_exists: Option<usize>,
    attempt_no: &mut usize,
    max_attempts: usize,
) -> Result<(), AzureStorageError> {
    if *attempt_no >= max_attempts {
        return Err(err);
    }

    match err {
        AzureStorageError::ContainerNotFound => {
            if create_container_if_not_exist {
                return super::create_container_if_not_exists(page_blob, max_attempts).await;
            } else {
                return Err(err);
            }
        }
        AzureStorageError::BlobNotFound => {
            if let Some(init_pages_amount) = create_blob_if_not_exists {
                super::create_blob_if_not_exists(page_blob, init_pages_amount, max_attempts)
                    .await?;

                return Ok(());
            } else {
                return Err(err);
            }
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
    *attempt_no += 1;

    Ok(())
}
