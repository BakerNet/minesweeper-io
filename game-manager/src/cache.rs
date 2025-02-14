use std::{
    future::Future,
    time::{Duration, Instant},
};

use tokio::sync::Mutex;

#[derive(Debug)]
pub struct CachedValue<T> {
    current: Mutex<Option<(Instant, T)>>,
    ttl: Duration,
}

impl<T: Clone> CachedValue<T> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            current: Mutex::new(None),
            ttl,
        }
    }

    #[allow(dead_code)]
    pub async fn get(&self) -> Option<T> {
        let mut current = self.current.lock().await;
        match &(*current) {
            None => None,
            Some(inner) => {
                if Instant::now() - self.ttl > inner.0 {
                    *current = None;
                    return None;
                }
                Some(inner.1.clone())
            }
        }
    }

    #[allow(dead_code)]
    pub async fn set(&self, value: T) {
        let mut current = self.current.lock().await;
        *current = Some((Instant::now(), value));
    }

    pub async fn get_or_set<F, Fut>(&self, f: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        let mut current = self.current.lock().await;
        match &(*current) {
            None => {
                let new = f().await;
                *current = Some((Instant::now(), new.clone()));
                new
            }
            Some(inner) => {
                if Instant::now() - self.ttl > inner.0 {
                    let new = f().await;
                    *current = Some((Instant::now(), new.clone()));
                    return new;
                }
                inner.1.clone()
            }
        }
    }
}
