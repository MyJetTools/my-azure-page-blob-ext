use my_azure_storage_sdk::{page_blob::AzurePageBlobStorage, AzureStorageError};

pub async fn create_container_if_not_exists(
    page_blob: &AzurePageBlobStorage,
    max_attempts: usize,
) -> Result<(), AzureStorageError> {
    let attempt_no = 0;
    loop {
        match page_blob.create_container_if_not_exist().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                super::handle_create_blob_or_container_error(err, attempt_no, max_attempts).await?;
            }
        }
    }
}
