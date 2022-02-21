use super::PagesInterval;

pub enum PageFromCacheResult {
    MissingInterval(PagesInterval),
    CachedInterval(PagesInterval),
}
