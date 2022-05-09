use std::sync::{Mutex, MutexGuard};

/// A sketchy way to not deal with mutex poisoning
///
/// **WARNING**: If the poison had occurred during a write, the data may be corrupted.
/// _If_ unsafe code had poisoned the mutex, memory corruption is possible
pub trait MutexExt<T> {
    fn force_lock(&self) -> MutexGuard<'_, T>;
}

impl<T> MutexExt<T> for Mutex<T> {
    fn force_lock(&self) -> MutexGuard<'_, T> {
        match self.lock() {
            Ok(guard) => guard,
            Err(error) => error.into_inner(),
        }
    }
}
