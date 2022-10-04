use my_azure_storage_sdk::{page_blob::AzurePageBlobStorage, AzureStorageError};

pub async fn resize(
    page_blob: &AzurePageBlobStorage,
    crate_container_if_not_exists: bool,
    create_blob_if_not_exists: bool,
    pages_amount: usize,
    max_attempts_no: usize,
) -> Result<(), AzureStorageError> {
    let mut attempt_no = 0;

    let auto_create_blob = if create_blob_if_not_exists {
        Some(pages_amount)
    } else {
        None
    };

    loop {
        match page_blob.resize(pages_amount).await {
            Ok(result) => {
                return Ok(result);
            }
            Err(err) => {
                super::handle_error(
                    page_blob,
                    err,
                    crate_container_if_not_exists,
                    auto_create_blob,
                    &mut attempt_no,
                    max_attempts_no,
                )
                .await?;
            }
        }
    }
}
