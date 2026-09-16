//! Recover inner guards from poisoned std locks so paint/event paths keep running.

use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub fn mutex<T>(lock: &Mutex<T>) -> MutexGuard<'_, T> {
    lock.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub fn read<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read().unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub fn write<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write().unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    fn poison_mutex<T>(lock: &Mutex<T>) {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = lock.lock().unwrap();
            panic!("poison");
        }));
    }

    fn poison_rwlock<T>(lock: &RwLock<T>) {
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = lock.write().unwrap();
            panic!("poison");
        }));
    }

    #[test]
    fn mutex_recovers_poison() {
        let lock = Mutex::new(3);
        poison_mutex(&lock);
        assert_eq!(*mutex(&lock), 3);
        *mutex(&lock) = 9;
        assert_eq!(*mutex(&lock), 9);
    }

    #[test]
    fn rwlock_recovers_poison() {
        let lock = RwLock::new(4);
        poison_rwlock(&lock);
        assert_eq!(*read(&lock), 4);
        *write(&lock) = 8;
        assert_eq!(*read(&lock), 8);
    }
}
