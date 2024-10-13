use std::time::{Duration, Instant};

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

    pub async fn get(&self) -> Option<T> {
        let mut value = self.current.lock().await;
        match &(*value) {
            None => None,
            Some(inner) => {
                if Instant::now() - self.ttl > inner.0 {
                    *value = None;
                    return None;
                }
                Some(inner.1.clone())
            }
        }
    }

    pub async fn set(&self, value: T) {
        let mut current = self.current.lock().await;
        *current = Some((Instant::now(), value));
    }
}
