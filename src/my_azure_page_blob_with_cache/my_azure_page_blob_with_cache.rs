use my_azure_storage_sdk::{
    page_blob::{consts::BLOB_PAGE_SIZE, MyAzurePageBlobStorage, PageBlobProperties},
    AzureStorageError,
};
use rust_extensions::AsSliceOrVec;
use tokio::sync::Mutex;

use crate::{FoundPages, PageBlobCachedData};

pub struct MyAzurePageBlobWithCache<
    TMyAzurePageBlobStorage: MyAzurePageBlobStorage + Send + Sync + 'static,
> {
    page_blob: TMyAzurePageBlobStorage,
    cache: Mutex<PageBlobCachedData>,
}

impl<TMyAzurePageBlobStorage: MyAzurePageBlobStorage + Send + Sync + 'static>
    MyAzurePageBlobWithCache<TMyAzurePageBlobStorage>
{
    pub fn new(page_blob: TMyAzurePageBlobStorage, pages_to_cache: usize) -> Self {
        Self {
            page_blob,
            cache: Mutex::new(PageBlobCachedData::new(pages_to_cache, BLOB_PAGE_SIZE)),
        }
    }
}

#[async_trait::async_trait]
impl<TMyAzurePageBlobStorage: MyAzurePageBlobStorage + Send + Sync + 'static> MyAzurePageBlobStorage
    for MyAzurePageBlobWithCache<TMyAzurePageBlobStorage>
{
    fn get_blob_name(&self) -> &str {
        self.page_blob.get_blob_name()
    }

    fn get_container_name(&self) -> &str {
        self.page_blob.get_container_name()
    }

    async fn create_container_if_not_exists(&self) -> Result<(), AzureStorageError> {
        self.page_blob.create_container_if_not_exists().await
    }

    async fn resize(&self, pages_amount: usize) -> Result<(), AzureStorageError> {
        let mut write_access = self.cache.lock().await;
        self.resize(pages_amount).await?;
        write_access.update_pages_amount(pages_amount);
        Ok(())
    }

    async fn create(&self, pages_amount: usize) -> Result<(), AzureStorageError> {
        self.page_blob.create(pages_amount).await
    }

    async fn get_pages(
        &self,
        start_page_no: usize,
        pages_amount: usize,
    ) -> Result<Vec<u8>, AzureStorageError> {
        let write_access = self.cache.lock().await;

        let mut found_pages = FoundPages::new(start_page_no, pages_amount);

        for page_no in start_page_no..start_page_no + pages_amount {
            if let Some(page) = write_access.pages_to_write.get_page(page_no) {
                found_pages.add(Some(page));
            } else if let Some(page) = &write_access.cached_pages {
                found_pages.add(page.get_payload(page_no));
            } else {
                found_pages.add(None);
            }
        }

        let payload: Vec<u8>;

        if let Some(pages_to_upload) = found_pages.get_pages_to_upload() {
            payload = self
                .page_blob
                .get_pages(pages_to_upload.from_page_no, pages_to_upload.amount)
                .await?;
            found_pages.upload_missing_pages(payload.as_slice());
        };

        Ok(found_pages.into_vec())
    }

    async fn save_pages<'s>(
        &self,
        start_page_no: usize,
        payload: impl Into<AsSliceOrVec<'s, u8>> + Send + Sync + 'static,
    ) -> Result<(), AzureStorageError> {
        let payload: AsSliceOrVec<'s, u8> = payload.into();
        let mut write_access = self.cache.lock().await;
        write_access
            .pages_to_write
            .update_pages(start_page_no, payload.into_vec());

        Ok(())
    }

    async fn delete(&self) -> Result<(), AzureStorageError> {
        self.page_blob.delete().await
    }
    async fn download(&self) -> Result<Vec<u8>, AzureStorageError> {
        self.page_blob.download().await
    }

    async fn get_blob_properties(&self) -> Result<PageBlobProperties, AzureStorageError> {
        let mut cache = self.cache.lock().await;
        if let Some(blob_properties) = &cache.page_blob_properties {
            return Ok(blob_properties.clone());
        }
        let page_blob_properties = self.page_blob.get_blob_properties().await?;

        cache.update_blob_properties(page_blob_properties.clone());

        Ok(page_blob_properties)
    }
}
