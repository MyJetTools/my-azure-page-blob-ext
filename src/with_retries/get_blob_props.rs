use my_azure_storage_sdk::{
    blob::BlobProperties, page_blob::AzurePageBlobStorage, AzureStorageError,
};

pub async fn get_blob_props(
    page_blob: &AzurePageBlobStorage,
    auto_create_container: bool,
    auto_create_blob: Option<usize>,
    max_attempts_no: usize,
) -> Result<BlobProperties, AzureStorageError> {
    let mut attempt_no = 0;
    loop {
        match page_blob.get_blob_properties().await {
            Ok(props) => {
                return Ok(props);
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
