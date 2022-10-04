use my_azure_storage_sdk::{
    page_blob::{consts::BLOB_PAGE_SIZE, AzurePageBlobStorage},
    AzureStorageError,
};

pub async fn save_pages(
    page_blob: &AzurePageBlobStorage,
    start_page_no: usize,
    payload: &[u8],
    crate_container_if_not_exists: bool,
    create_blob_if_not_exists: bool,
    max_attempts_no: usize,
) -> Result<(), AzureStorageError> {
    let mut attempt_no = 0;

    let auto_create_blob = if create_blob_if_not_exists {
        let page_blob_size = start_page_no + payload.len() / BLOB_PAGE_SIZE;
        Some(page_blob_size)
    } else {
        None
    };

    loop {
        match page_blob.save_pages(start_page_no, payload.to_vec()).await {
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
