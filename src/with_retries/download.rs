use my_azure_storage_sdk::{page_blob::AzurePageBlobStorage, AzureStorageError};

pub async fn download(
    page_blob: &AzurePageBlobStorage,
    auto_create_container: bool,
    auto_create_blob: Option<usize>,
    max_attempts_no: usize,
) -> Result<Vec<u8>, AzureStorageError> {
    let mut attempt_no = 0;
    loop {
        match page_blob.download().await {
            Ok(result) => {
                return Ok(result);
            }
            Err(err) => {
                super::handle_read_error(
                    page_blob,
                    err,
                    auto_create_container,
                    auto_create_blob,
                    &mut attempt_no,
                    max_attempts_no,
                )
                .await?;
            }
        }
    }
}
