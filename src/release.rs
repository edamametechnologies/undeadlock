use dashmap::DashMap;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use tokio::sync::{Mutex, RwLock};

#[derive(Default)]
pub struct LockContextGuard {
    _private: (),
}

impl Drop for LockContextGuard {
    fn drop(&mut self) {}
}

pub fn set_lock_context(_context: impl Into<String>) -> LockContextGuard {
    LockContextGuard { _private: () }
}

pub fn clear_lock_context() {}

pub type CustomRwLock<T> = RwLock<T>;
pub type CustomMutex<T> = Mutex<T>;

/// Thin wrapper around `DashMap` that preserves the `CustomDashMap::new` /
/// `new_with_timeout` constructors used throughout the codebase. Every other
/// method call is forwarded via `Deref`/`DerefMut`.
pub struct CustomDashMap<K, V>(DashMap<K, V>);

impl<K: Eq + Hash, V> CustomDashMap<K, V> {
    pub fn new(_name: &str) -> Self {
        Self(DashMap::new())
    }

    pub fn new_with_timeout(_name: &str, _write_lock_timeout_secs: u64) -> Self {
        Self::new(_name)
    }

    pub fn inner(&self) -> &DashMap<K, V> {
        &self.0
    }

    pub fn drain(&self) -> std::vec::IntoIter<(K, V)>
    where
        K: Clone,
        V: Clone,
    {
        let items: Vec<(K, V)> = self
            .0
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();
        self.0.clear();
        items.into_iter()
    }

    pub fn dump_lock_state(&self) {}
}

impl<K, V> Deref for CustomDashMap<K, V> {
    type Target = DashMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V> DerefMut for CustomDashMap<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<K, V> Default for CustomDashMap<K, V>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Self(DashMap::new())
    }
}

impl<K, V> std::fmt::Debug for CustomDashMap<K, V>
where
    K: Eq + Hash + std::fmt::Debug,
    V: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomDashMap")
            .field("len", &self.0.len())
            .finish()
    }
}
