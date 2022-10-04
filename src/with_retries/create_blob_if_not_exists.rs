use my_azure_storage_sdk::{page_blob::AzurePageBlobStorage, AzureStorageError};

pub async fn create_blob_if_not_exists(
    page_blob: &AzurePageBlobStorage,
    init_pages_amount: usize,
    max_attempts: usize,
) -> Result<usize, AzureStorageError> {
    let attempt_no = 0;
    loop {
        match page_blob.create_if_not_exists(init_pages_amount).await {
            Ok(result) => return Ok(result),
            Err(err) => {
                super::handle_create_blob_or_container_error(err, attempt_no, max_attempts).await?;
            }
        }
    }
}
