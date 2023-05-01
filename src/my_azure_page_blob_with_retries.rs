use std::time::Duration;

use my_azure_storage_sdk::{
    page_blob::{AzurePageBlobStorage, MyAzurePageBlobStorage, PageBlobProperties},
    AzureStorageError,
};
use rust_extensions::AsSliceOrVec;

pub struct MyAzurePageBlobStorageWithRetries {
    pub page_blob: AzurePageBlobStorage,
    pub retries_amount: usize,
    pub retry_delay: Duration,
}

impl MyAzurePageBlobStorageWithRetries {
    pub fn new(
        page_blob: AzurePageBlobStorage,
        retries_amount: usize,
        retry_delay: Duration,
    ) -> Self {
        Self {
            page_blob,
            retries_amount,
            retry_delay,
        }
    }
}

#[async_trait::async_trait]
impl MyAzurePageBlobStorage for MyAzurePageBlobStorageWithRetries {
    fn get_blob_name(&self) -> &str {
        self.page_blob.get_blob_name()
    }

    fn get_container_name(&self) -> &str {
        self.page_blob.get_container_name()
    }

    async fn resize(&self, pages_amount: usize) -> Result<(), AzureStorageError> {
        let mut attempt_no = 0;

        loop {
            match self.page_blob.resize(pages_amount).await {
                Ok(result) => {
                    return Ok(result);
                }
                Err(err) => {
                    if attempt_no >= self.retries_amount {
                        return Err(err);
                    }
                    attempt_no += 1;

                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }
    }

    async fn create_container_if_not_exists(&self) -> Result<(), AzureStorageError> {
        let mut attempt_no = 0;

        loop {
            match self.page_blob.create_container_if_not_exists().await {
                Ok(result) => {
                    return Ok(result);
                }
                Err(err) => {
                    if attempt_no >= self.retries_amount {
                        return Err(err);
                    }
                    attempt_no += 1;

                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }
    }

    async fn create(&self, pages_amount: usize) -> Result<(), AzureStorageError> {
        let mut attempt_no = 0;

        loop {
            match self.page_blob.create(pages_amount).await {
                Ok(result) => {
                    return Ok(result);
                }
                Err(err) => {
                    if attempt_no >= self.retries_amount {
                        return Err(err);
                    }
                    attempt_no += 1;

                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }
    }
    async fn create_if_not_exists(
        &self,
        pages_amount: usize,
        auto_create_container: bool,
    ) -> Result<PageBlobProperties, AzureStorageError> {
        let mut attempt_no = 0;

        loop {
            match self
                .page_blob
                .create_if_not_exists(pages_amount, auto_create_container)
                .await
            {
                Ok(result) => {
                    return Ok(result);
                }
                Err(err) => {
                    if attempt_no >= self.retries_amount {
                        return Err(err);
                    }
                    attempt_no += 1;

                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }
    }
    async fn get_pages(
        &self,
        start_page_no: usize,
        pages_amount: usize,
    ) -> Result<Vec<u8>, AzureStorageError> {
        let mut attempt_no = 0;

        loop {
            match self.page_blob.get_pages(start_page_no, pages_amount).await {
                Ok(result) => {
                    return Ok(result);
                }
                Err(err) => {
                    if attempt_no >= self.retries_amount {
                        return Err(err);
                    }
                    attempt_no += 1;

                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }
    }

    async fn save_pages<'s>(
        &self,
        start_page_no: usize,
        payload: impl Into<AsSliceOrVec<'s, u8>> + Send + Sync + 'static,
    ) -> Result<(), AzureStorageError> {
        let mut attempt_no = 0;

        let payload: AsSliceOrVec<'s, u8> = payload.into();

        let payload = payload.as_slice();

        loop {
            match self.page_blob.save_pages(start_page_no, payload).await {
                Ok(result) => {
                    return Ok(result);
                }
                Err(err) => {
                    if attempt_no >= self.retries_amount {
                        return Err(err);
                    }
                    attempt_no += 1;

                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }
    }

    async fn delete(&self) -> Result<(), AzureStorageError> {
        let mut attempt_no = 0;

        loop {
            match self.page_blob.delete().await {
                Ok(result) => {
                    return Ok(result);
                }
                Err(err) => {
                    if attempt_no >= self.retries_amount {
                        return Err(err);
                    }
                    attempt_no += 1;

                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }
    }

    async fn download(&self) -> Result<Vec<u8>, AzureStorageError> {
        let mut attempt_no = 0;

        loop {
            match self.page_blob.download().await {
                Ok(result) => {
                    return Ok(result);
                }
                Err(err) => {
                    if attempt_no >= self.retries_amount {
                        return Err(err);
                    }
                    attempt_no += 1;

                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }
    }

    async fn get_blob_properties(&self) -> Result<PageBlobProperties, AzureStorageError> {
        let mut attempt_no = 0;

        loop {
            match self.page_blob.get_blob_properties().await {
                Ok(result) => {
                    return Ok(result.into());
                }
                Err(err) => {
                    if attempt_no >= self.retries_amount {
                        return Err(err);
                    }
                    attempt_no += 1;

                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }
    }
}
