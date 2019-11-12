use crate::common::{PoisonableCriticalSection, POISONABLE_ZEROED};
use crate::EnteredCritical;

use crate::wrapper::{
    delete_cs, enter_cs, init_cs, init_cs_with_spin_count, set_cs_spin_count, try_enter_cs,
};

use winapi::um::minwinbase::CRITICAL_SECTION;

use std::{
    fmt::{self, Formatter},
    sync::Arc,
};

#[derive(Clone)]
pub struct CriticalSection {
    inner: Arc<PoisonableCriticalSection>,
}

// Safety: *CRITICAL_SECTION aka lpCriticalSection (effectivity provided by Arc) is Send.
// Critical Section API is naturally Sync.
unsafe impl Send for CriticalSection {}
unsafe impl Sync for CriticalSection {}

impl PartialEq for CriticalSection {
    fn eq(&self, other: &Self) -> bool {
        &*self.inner as *const PoisonableCriticalSection == &*other.inner as *const _
    }
}
impl Eq for CriticalSection {}

impl CriticalSection {
    pub fn new() -> Self {
        let inner = Arc::new(POISONABLE_ZEROED);
        let ptr = &inner.critical as *const _ as *mut CRITICAL_SECTION;
        // Safety: ptr is to a brand new CRITICAL_SECTION object that
        // will not be moved in memory. Might panic.
        unsafe {
            init_cs(ptr);
        }
        Self { inner }
    }
    pub fn with_spin_count(spin_count: u32) -> Self {
        let inner = Arc::new(POISONABLE_ZEROED);
        let ptr = &inner.critical as *const _ as *mut CRITICAL_SECTION;
        // Safety: Never fails, and ptr is to a brand new
        // CRITICAL_SECTION object that will not be moved in memory.
        unsafe {
            init_cs_with_spin_count(ptr, spin_count);
        }
        Self { inner }
    }
    #[allow(non_snake_case)]
    fn lpCriticalSection(&self) -> *mut CRITICAL_SECTION {
        self.inner.critical.get()
    }
    pub fn enter<'c>(&'c self) -> EnteredCritical<'c> {
        // Safety: might panic, no return value. Naturally thread-safe.
        unsafe { enter_cs(self.lpCriticalSection()) }
        EnteredCritical::new(&self.inner)
    }
    pub fn try_enter<'c>(&'c self) -> Option<EnteredCritical<'c>> {
        // Safety: returns non-zero if we are in critical section when call returns.
        // Naturally thread-safe.
        match unsafe { try_enter_cs(self.lpCriticalSection()) } {
            0 => None,
            _ => Some(EnteredCritical::new(&self.inner)),
        }
    }
    pub fn set_spin_count(&self, spin_count: u32) -> u32 {
        // Safety: cannot fail. Returns previous spin_count. Naturally thread-safe.
        unsafe { set_cs_spin_count(self.lpCriticalSection(), spin_count) }
    }
}

impl Drop for CriticalSection {
    fn drop(&mut self) {
        if Arc::strong_count(&self.inner) == 1 {
            // Safety: we have exclusive access by knowing strong count is one in drop,
            // we never created any weak refs, and FFI call never fails
            unsafe {
                delete_cs(self.lpCriticalSection());
            }
        }
    }
}

impl fmt::Debug for CriticalSection {
    fn fmt(&self, out: &mut Formatter) -> fmt::Result {
        write!(out, "CriticalSection: {:p}", self.inner)
    }
}

#[cfg(test)]
mod tests {
    use crate::CriticalSection;
    use std::thread;

    /*
        🎶 99 Mutating Threads on the Wall 🎶
              🎶 99 Mutating Threads 🎶
         🎶 Take One Down - Unwind it Now 🎶
        🎶 98 Mutating Threads on the Wall 🎶
    */

    #[test]
    fn threads_on_the_wall() {
        static mut X: usize = 0;
        let mut handles = Vec::with_capacity(99);
        let critical = CriticalSection::new();
        for i in 0..99 {
            let crit = critical.clone();
            handles.push(thread::spawn(move || {
                let entered = crit.enter();
                if i == 0 {
                    panic!("Take one down")
                }
                let x = 1 + unsafe { X };
                thread::yield_now();
                unsafe {
                    X = x;
                }
                entered.leave();
            }));
        }
        for (i, handle) in handles.into_iter().enumerate() {
            if i == 0 {
                handle.join().unwrap_err();
            } else {
                handle.join().unwrap();
            }
        }
        assert_eq!(98, unsafe { X });
        assert!(critical.enter().is_poisoned());
    }

    #[test]
    fn clone_eq() {
        let c1 = CriticalSection::new();
        let c2 = c1.clone();
        assert_eq!(c1, c2);
    }
}
