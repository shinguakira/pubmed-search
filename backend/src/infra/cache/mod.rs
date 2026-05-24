//! In-memory article cache. Lives in the server process for as long as
//! it's up — no DB, no disk, no eviction (Phase 2 will move this to
//! sqlx + SQLite with a TTL).
//!
//! Used by:
//! * `/api/search?bulk=true` — populates the cache with every
//!   ArticleDetail it fetches via `efetch_bulk`.
//! * `/api/article/{pmid}` — checks the cache before hitting NCBI.
//!
//! This is how the bulk-search speedup actually lands at the user: a
//! subsequent article-detail call is served from this map in ~0 ms
//! instead of ~600 ms.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::infra::ncbi::ArticleDetail;

#[derive(Clone, Default)]
pub struct ArticleCache {
    inner: Arc<RwLock<HashMap<String, ArticleDetail>>>,
}

impl ArticleCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, pmid: &str) -> Option<ArticleDetail> {
        let guard = self.inner.read().ok()?;
        guard.get(pmid).cloned()
    }

    pub fn put(&self, article: ArticleDetail) {
        if let Ok(mut guard) = self.inner.write() {
            guard.insert(article.pmid.clone(), article);
        }
    }

    pub fn put_many<I: IntoIterator<Item = ArticleDetail>>(&self, articles: I) {
        if let Ok(mut guard) = self.inner.write() {
            for a in articles {
                guard.insert(a.pmid.clone(), a);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.inner.read().map(|g| g.len()).unwrap_or(0)
    }
}
