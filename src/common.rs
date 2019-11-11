use crate::wrapper::{
    leave_cs,
    set_cs_spin_count,
};

use static_assertions::assert_not_impl_all;

use winapi::um::{
    minwinbase::CRITICAL_SECTION,
    // synchapi::{
    //     LeaveCriticalSection,
    //     SetCriticalSectionSpinCount,
    // },
};

pub(crate) const CRIT_ZEROED: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: 0 as *mut _,
    LockCount: 0,
    LockSemaphore: 0 as *mut _,
    OwningThread: 0 as *mut _,
    RecursionCount: 0,
    SpinCount: 0
};

pub struct EnteredCritical<'c>(&'c CRITICAL_SECTION);

// Safety: it is not okay to enter from one thread and leave from another.
assert_not_impl_all!(EnteredCritical: Send, Sync);

impl<'c> EnteredCritical<'c> {
    pub(crate) fn new(ptr: &'c CRITICAL_SECTION) -> Self {
        Self(ptr)
    }
}

impl EnteredCritical<'_> {
    #[allow(non_snake_case)]
    fn lpCriticalSection(&self) -> *mut CRITICAL_SECTION {
        self.0 as *const _ as *mut _
    }
    pub fn leave(self) {
        drop(self)
    }
    pub fn set_spin_count(&self, spin_count: u32) -> u32 {
        // Safety: cannot fail. Returns previous spin_count. Naturally thread-safe.
        unsafe { set_cs_spin_count(self.lpCriticalSection(), spin_count) }
    }
}

impl Drop for EnteredCritical<'_> {
    fn drop(&mut self) {
        unsafe {
            leave_cs(self.lpCriticalSection())
        }
    }
}