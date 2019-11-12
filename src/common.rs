use crate::wrapper::{leave_cs, set_cs_spin_count};

use static_assertions::assert_not_impl_all;

use winapi::um::minwinbase::CRITICAL_SECTION;

use core::cell::UnsafeCell;

pub(crate) const CRIT_ZEROED: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: 0 as *mut _,
    LockCount: 0,
    LockSemaphore: 0 as *mut _,
    OwningThread: 0 as *mut _,
    RecursionCount: 0,
    SpinCount: 0,
};

pub(crate) struct PoisonableCriticalSection {
    pub critical: UnsafeCell<CRITICAL_SECTION>,
    poison: UnsafeCell<bool>,
}

impl PoisonableCriticalSection {
    pub(crate) unsafe fn clear_poison_unsynced(&self) {
        self.poison.get().write(false)
    }
}

pub(crate) const POISONABLE_ZEROED: PoisonableCriticalSection = PoisonableCriticalSection {
    critical: UnsafeCell::new(CRIT_ZEROED),
    poison: UnsafeCell::new(false),
};

pub struct EnteredCritical<'c>(&'c PoisonableCriticalSection);

// Safety: it is not okay to enter from one thread and leave from another, or leave twice.
assert_not_impl_all!(EnteredCritical: Send, Sync, Copy, Clone);

impl<'c> EnteredCritical<'c> {
    pub(crate) fn new(ptr: &'c PoisonableCriticalSection) -> Self {
        Self(ptr)
    }
}

impl EnteredCritical<'_> {
    #[allow(non_snake_case)]
    fn lpCriticalSection(&self) -> *mut CRITICAL_SECTION {
        self.0.critical.get()
    }
    pub fn leave(self) {
        drop(self)
    }
    pub fn set_spin_count(&self, spin_count: u32) -> u32 {
        // Safety: cannot fail. Returns previous spin_count. Naturally thread-safe.
        unsafe { set_cs_spin_count(self.lpCriticalSection(), spin_count) }
    }
    pub fn is_poisoned(&self) -> bool {
        // Safety: can only read or write poison value while entered
        unsafe { self.0.poison.get().read() }
    }
    pub fn clear_poison(&self) {
        // Safety: can only read or write poison value while entered
        unsafe { self.0.poison.get().write(false) }
    }
}

impl Drop for EnteredCritical<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            // Safety: can only read or write poison value while entered
            unsafe { self.0.poison.get().write(true) }
        }
        // Safety: Cannot fail, no return value, leave exactly once.
        unsafe { leave_cs(self.lpCriticalSection()) }
    }
}
