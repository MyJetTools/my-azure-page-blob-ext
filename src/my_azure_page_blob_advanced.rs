use std::time::Duration;

use my_azure_storage_sdk::{
    blob::BlobProperties,
    page_blob::{consts::BLOB_PAGE_SIZE, AzurePageBlobStorage},
    AzureStorageError,
};
use tokio::sync::Mutex;

use crate::{
    exact_payload::ExactPayloadPositions, page_blob_cached_data::PageBlobCachedData,
    pages_cache::PageFromCacheResult, ExactPayload,
};

pub struct MyAzurePageBlobAdvanced {
    page_blob: AzurePageBlobStorage,
    retries_attempts_amount: usize,
    delay_between_attempts: Duration,
    pages_to_save_per_roundtrip: usize,
    cache: Mutex<PageBlobCachedData>,
}

impl MyAzurePageBlobAdvanced {
    pub fn new(
        page_blob: AzurePageBlobStorage,
        retries_attempts_amount: usize,
        delay_between_attempts: Duration,
        pages_to_save_per_roundtrip: usize,
        pages_to_cache: usize,
    ) -> Self {
        Self {
            page_blob,
            retries_attempts_amount,
            delay_between_attempts,
            pages_to_save_per_roundtrip,
            cache: Mutex::new(PageBlobCachedData::new(pages_to_cache, BLOB_PAGE_SIZE)),
        }
    }

    pub fn get_blob_name(&self) -> &str {
        self.page_blob.get_blob_name()
    }

    pub fn get_container_name(&self) -> &str {
        self.page_blob.get_container_name()
    }

    async fn get_pages_amount(
        &self,
        cache: &mut PageBlobCachedData,
    ) -> Result<usize, AzureStorageError> {
        if let Some(size) = cache.get_pages_amount() {
            return Ok(size);
        }

        let props = self.execute_get_blob_properties(cache).await?;

        return Ok(props.blob_size / BLOB_PAGE_SIZE);
    }

    async fn execute_resize(
        &self,
        pages_amount: usize,
        cache: &mut PageBlobCachedData,
    ) -> Result<(), AzureStorageError> {
        let mut attempt_no = 0;

        loop {
            match self.page_blob.resize(pages_amount).await {
                Ok(result) => {
                    cache.set_pages_amount(pages_amount);
                    return Ok(result);
                }
                Err(err) => {
                    attempt_no += 1;
                    self.handle_error(err, attempt_no, cache).await?;
                }
            }
        }
    }

    pub async fn resize(&self, pages_amount: usize) -> Result<(), AzureStorageError> {
        let mut write_access = self.cache.lock().await;
        self.execute_resize(pages_amount, &mut write_access).await
    }

    pub async fn create_container_if_not_exist(&self) -> Result<(), AzureStorageError> {
        let mut cache = self.cache.lock().await;

        let mut attempt_no = 0;

        loop {
            match self.page_blob.create_container_if_not_exist().await {
                Ok(result) => {
                    cache.set_pages_amount(0);
                    return Ok(result);
                }
                Err(err) => {
                    attempt_no += 1;
                    self.handle_error(err, attempt_no, &mut cache).await?;
                }
            }
        }
    }

    pub async fn get_available_pages_amount(&self) -> Result<usize, AzureStorageError> {
        let mut cache = self.cache.lock().await;
        return self.get_pages_amount(&mut cache).await;
    }

    pub async fn create(&self, pages_amount: usize) -> Result<(), AzureStorageError> {
        let mut attempt_no = 0;

        let mut cache = self.cache.lock().await;

        loop {
            match self.page_blob.create(pages_amount).await {
                Ok(result) => {
                    cache.set_pages_amount(pages_amount);
                    return Ok(result);
                }
                Err(err) => {
                    attempt_no += 1;
                    self.handle_error(err, attempt_no, &mut cache).await?;
                }
            }
        }
    }

    pub async fn create_if_not_exists(
        &self,
        pages_amount: usize,
    ) -> Result<usize, AzureStorageError> {
        let mut attempt_no = 0;

        let mut cache = self.cache.lock().await;

        loop {
            match self.page_blob.create_if_not_exists(pages_amount).await {
                Ok(result) => {
                    cache.set_pages_amount(pages_amount);
                    return Ok(result);
                }
                Err(err) => {
                    attempt_no += 1;
                    self.handle_error(err, attempt_no, &mut cache).await?;
                }
            }
        }
    }

    pub async fn get(
        &self,
        start_page_no: usize,
        pages_amount: usize,
    ) -> Result<Vec<u8>, AzureStorageError> {
        let mut cache = self.cache.lock().await;
        self.execute_get(start_page_no, pages_amount, &mut cache)
            .await
    }

    async fn execute_get(
        &self,
        start_page_no: usize,
        pages_amount: usize,
        cache: &mut PageBlobCachedData,
    ) -> Result<Vec<u8>, AzureStorageError> {
        if cache.cached_pages.is_some() {
            return self
                .execute_get_using_cache(start_page_no, pages_amount, cache)
                .await;
        } else {
            return self
                .execute_get_not_using_cache(start_page_no, pages_amount, cache)
                .await;
        }
    }

    async fn execute_get_not_using_cache(
        &self,
        start_page_no: usize,
        pages_amount: usize,
        cache: &mut PageBlobCachedData,
    ) -> Result<Vec<u8>, AzureStorageError> {
        let mut attempt_no = 0;

        loop {
            match self.page_blob.get(start_page_no, pages_amount).await {
                Ok(result) => {
                    if let Some(cached_pages) = &mut cache.cached_pages {
                        cached_pages.save_to_cache(start_page_no, result.as_slice());
                    }
                    return Ok(result);
                }
                Err(err) => {
                    attempt_no += 1;
                    self.handle_error(err, attempt_no, cache).await?;
                }
            }
        }
    }

    async fn execute_get_using_cache(
        &self,
        start_page_no: usize,
        pages_amount: usize,
        cache: &mut PageBlobCachedData,
    ) -> Result<Vec<u8>, AzureStorageError> {
        let data_from_cache = cache
            .cached_pages
            .as_ref()
            .unwrap()
            .get(start_page_no, pages_amount);

        let mut result = Vec::with_capacity(pages_amount * BLOB_PAGE_SIZE);
        unsafe {
            result.set_len(result.capacity());
        }

        let mut result_offset = 0;

        for interval in data_from_cache {
            match &interval {
                PageFromCacheResult::MissingInterval(interval) => {
                    let interval_size = interval.pages_amount * BLOB_PAGE_SIZE;
                    let chunk = self
                        .execute_get_not_using_cache(
                            interval.start_page_no,
                            interval.pages_amount,
                            cache,
                        )
                        .await?;

                    let slice_to_copy = &mut result[result_offset..result_offset + interval_size];

                    slice_to_copy.copy_from_slice(chunk.as_slice());
                    result_offset += interval_size;
                }
                PageFromCacheResult::CachedInterval(interval) => {
                    for page_no in
                        interval.start_page_no..interval.start_page_no + interval.pages_amount
                    {
                        let payload_from_cache = cache
                            .cached_pages
                            .as_ref()
                            .unwrap()
                            .get_payload(page_no)
                            .unwrap();

                        let slice_to_copy = &mut result[result_offset..BLOB_PAGE_SIZE];

                        slice_to_copy.copy_from_slice(payload_from_cache);

                        result_offset += BLOB_PAGE_SIZE;
                    }
                }
            }
        }

        Ok(result)
    }

    pub async fn save_pages(
        &self,
        start_page_no: usize,
        payload: Vec<u8>,
    ) -> Result<(), AzureStorageError> {
        let mut cache = self.cache.lock().await;

        self.execute_save_pages(start_page_no, payload, &mut cache)
            .await?;

        Ok(())
    }

    pub async fn execute_save_pages(
        &self,
        start_page_no: usize,
        payload: Vec<u8>,
        cache: &mut PageBlobCachedData,
    ) -> Result<(), AzureStorageError> {
        let pages_to_write = payload.len() / BLOB_PAGE_SIZE;

        if pages_to_write <= self.pages_to_save_per_roundtrip {
            self.save_pages_as_one_shot(start_page_no, payload.to_vec(), cache)
                .await?
        } else {
            self.save_pages_per_several_shots(
                start_page_no,
                payload.to_vec(),
                pages_to_write,
                cache,
            )
            .await?
        };

        if let Some(cached_pages) = &mut cache.cached_pages {
            cached_pages.save_to_cache(start_page_no, payload.as_slice());
        }

        Ok(())
    }
    async fn save_pages_as_one_shot(
        &self,
        start_page_no: usize,
        payload: Vec<u8>,
        cache: &mut PageBlobCachedData,
    ) -> Result<(), AzureStorageError> {
        let mut attempt_no = 0;

        loop {
            match self
                .page_blob
                .save_pages(start_page_no, payload.to_vec())
                .await
            {
                Ok(result) => {
                    return Ok(result);
                }
                Err(err) => {
                    attempt_no += 1;
                    self.handle_error(err, attempt_no, cache).await?;
                }
            }
        }
    }

    async fn save_pages_per_several_shots(
        &self,
        start_page_no: usize,
        payload: Vec<u8>,
        pages_to_write: usize,
        cache: &mut PageBlobCachedData,
    ) -> Result<(), AzureStorageError> {
        let mut remain_pages_to_write = pages_to_write;

        let mut start_page_no = start_page_no;
        let mut buffer_offset = 0;

        while remain_pages_to_write >= self.pages_to_save_per_roundtrip {
            let buffer_to_send = &payload
                [buffer_offset..buffer_offset + self.pages_to_save_per_roundtrip * BLOB_PAGE_SIZE];

            self.save_pages_as_one_shot(start_page_no, buffer_to_send.to_vec(), cache)
                .await?;

            start_page_no += self.pages_to_save_per_roundtrip;
            buffer_offset += self.pages_to_save_per_roundtrip * BLOB_PAGE_SIZE;
            remain_pages_to_write -= self.pages_to_save_per_roundtrip;
        }

        if remain_pages_to_write > 0 {
            let buffer_to_send = &payload
                [buffer_offset..buffer_offset + self.pages_to_save_per_roundtrip * BLOB_PAGE_SIZE];

            self.save_pages_as_one_shot(start_page_no, buffer_to_send.to_vec(), cache)
                .await?;
        }

        Ok(())
    }

    pub async fn delete(&self) -> Result<(), AzureStorageError> {
        let mut attempt_no = 0;

        let mut cache = self.cache.lock().await;

        loop {
            match self.page_blob.delete().await {
                Ok(result) => {
                    cache.reset_cache();
                    return Ok(result);
                }
                Err(err) => {
                    attempt_no += 1;
                    self.handle_error(err, attempt_no, &mut cache).await?;
                }
            }
        }
    }

    pub async fn delete_if_exists(&self) -> Result<(), AzureStorageError> {
        let mut attempt_no = 0;

        let mut cache = self.cache.lock().await;

        loop {
            match self.page_blob.delete_if_exists().await {
                Ok(result) => {
                    cache.reset_cache();
                    return Ok(result);
                }
                Err(err) => {
                    attempt_no += 1;
                    self.handle_error(err, attempt_no, &mut cache).await?;
                }
            }
        }
    }

    pub async fn download(&self) -> Result<Vec<u8>, AzureStorageError> {
        let mut attempt_no = 0;
        let mut cache = self.cache.lock().await;

        loop {
            match self.page_blob.download().await {
                Ok(result) => {
                    return Ok(result);
                }
                Err(err) => {
                    attempt_no += 1;
                    self.handle_error(err, attempt_no, &mut cache).await?;
                }
            }
        }
    }

    async fn execute_get_blob_properties(
        &self,
        cache: &mut PageBlobCachedData,
    ) -> Result<BlobProperties, AzureStorageError> {
        let mut attempt_no = 0;

        loop {
            match self.page_blob.get_blob_properties().await {
                Ok(result) => {
                    cache.set_pages_amount(result.blob_size / BLOB_PAGE_SIZE);
                    return Ok(result);
                }
                Err(err) => {
                    attempt_no += 1;
                    self.handle_error(err, attempt_no, cache).await?;
                }
            }
        }
    }

    pub async fn get_blob_properties(&self) -> Result<BlobProperties, AzureStorageError> {
        let mut cache = self.cache.lock().await;
        self.execute_get_blob_properties(&mut cache).await
    }

    pub async fn get_payload(
        &self,
        start_pos: usize,
        read_size: usize,
    ) -> Result<ExactPayload, AzureStorageError> {
        let positions = ExactPayloadPositions::new(start_pos, read_size, BLOB_PAGE_SIZE);

        let mut write_access = self.cache.lock().await;

        let blob_size = self.get_pages_amount(&mut write_access).await? * BLOB_PAGE_SIZE;

        if blob_size < positions.end_pos {
            return Err(AzureStorageError::UnknownError {
                msg: format!(
                    "Range is violated. BlobSize: {}. StartPos:{}, ReadSize:{}",
                    blob_size, start_pos, read_size
                ),
            });
        }

        let positions = ExactPayloadPositions::new(start_pos, read_size, BLOB_PAGE_SIZE);

        let payload = self
            .execute_get(
                positions.start_page_no,
                positions.get_full_pages_amount(),
                &mut write_access,
            )
            .await?;

        let result = ExactPayload {
            payload,
            offset: positions.get_payload_offset(),
            size: read_size,
        };

        Ok(result)
    }

    pub async fn save_payload(
        &self,
        start_pos: usize,
        payload: &[u8],
        auto_ressize_rate: Option<usize>,
    ) -> Result<(), AzureStorageError> {
        let positions = ExactPayloadPositions::new(start_pos, payload.len(), BLOB_PAGE_SIZE);

        let mut cache = self.cache.lock().await;

        let blob_size = self.get_pages_amount(&mut cache).await?;

        if blob_size < positions.end_pos {
            if let Some(auto_ressize_rate) = auto_ressize_rate {
                let pages_to_ressize = crate::utils::calc_pages_amount_to_ressize(
                    positions.end_pos,
                    BLOB_PAGE_SIZE,
                    auto_ressize_rate,
                );

                self.execute_resize(pages_to_ressize, &mut cache).await?;
            } else {
                return Err(AzureStorageError::UnknownError {
                    msg: format!(
                        "Range is violated. BlobSize: {}. StartPos:{}, WriteSize:{}",
                        blob_size,
                        start_pos,
                        payload.len()
                    ),
                });
            }
        }

        let mut blob_payload = self
            .execute_get(
                positions.start_page_no,
                positions.get_full_pages_amount(),
                &mut cache,
            )
            .await?;

        let dest = &mut blob_payload
            [positions.get_payload_offset()..positions.get_payload_offset() + payload.len()];

        dest.copy_from_slice(payload);

        self.execute_save_pages(positions.start_page_no, blob_payload, &mut cache)
            .await?;

        Ok(())
    }

    async fn handle_error(
        &self,
        err: AzureStorageError,
        attempt_no: usize,
        cache: &mut PageBlobCachedData,
    ) -> Result<(), AzureStorageError> {
        match err {
            AzureStorageError::ContainerNotFound => Err(err),
            AzureStorageError::BlobNotFound => Err(err),
            AzureStorageError::BlobAlreadyExists => Err(err),
            AzureStorageError::ContainerBeingDeleted => Err(err),
            AzureStorageError::ContainerAlreadyExists => Err(err),
            AzureStorageError::InvalidPageRange => Err(err),
            AzureStorageError::RequestBodyTooLarge => Err(err),
            AzureStorageError::UnknownError { msg } => Err(AzureStorageError::UnknownError { msg }),
            AzureStorageError::InvalidResourceName => Err(AzureStorageError::InvalidResourceName),
            AzureStorageError::IoError(err) => Err(AzureStorageError::IoError(err)),

            AzureStorageError::Timeout => todo!(),

            AzureStorageError::HyperError(err) => {
                println!("Hyper error. Attempt: {}. Err:{}", attempt_no, err);

                if attempt_no >= self.retries_attempts_amount {
                    cache.reset_cache();
                    Err(AzureStorageError::HyperError(err))
                } else {
                    tokio::time::sleep(self.delay_between_attempts).await;
                    Ok(())
                }
            }
        }
    }
}

#[cfg(test)]
mod test {

    use std::sync::Arc;

    use my_azure_storage_sdk::AzureStorageConnection;

    use super::*;

    #[tokio::test]
    async fn test_exact_amount() {
        let connection = AzureStorageConnection::new_in_memory();
        let page_blob_mock =
            AzurePageBlobStorage::new(Arc::new(connection), "test".to_string(), "test".to_string())
                .await;

        page_blob_mock
            .create_container_if_not_exist()
            .await
            .unwrap();

        page_blob_mock.create(0).await.unwrap();

        let page_blob =
            MyAzurePageBlobAdvanced::new(page_blob_mock, 3, Duration::from_secs(3), 10, 10);

        page_blob
            .save_payload(3, vec![3u8, 4u8, 5u8, 6u8].as_slice(), Some(1))
            .await
            .unwrap();

        let result = page_blob.download().await.unwrap();

        assert_eq!(
            [0u8, 0u8, 0u8, 3u8, 4u8, 5u8, 6u8, 0u8].as_slice(),
            &result[..8]
        );

        let read_result = page_blob.get_payload(4, 3).await.unwrap();

        assert_eq!([4u8, 5u8, 6u8].as_slice(), read_result.as_slice());
    }
}
