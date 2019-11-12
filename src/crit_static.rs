// This module should not use std.
// A synchronization primitive like this is desireable
// in low-level things such as allocators. In the
// specific case of allocators, they will generally
// need to avoid using a type which itself allocates.
// By not using std, we are certain to not allocate.

use crate::common::{PoisonableCriticalSection, POISONABLE_ZEROED};
use crate::EnteredCritical;

use crate::wrapper::{
    delete_cs, enter_cs, init_cs, init_cs_with_spin_count, set_cs_spin_count, try_enter_cs,
};

use winapi::um::minwinbase::CRITICAL_SECTION;

use core::sync::atomic::{AtomicUsize, Ordering};

const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;
const POISONED: usize = 3;

/// CriticalStatic is a CriticalStatic primitive that can be contructed statically and safely used.
/// Deleting a CriticalStatic is unsafe, and you must either ensure it gets re-initialized prior to
/// being used elsewhere, or else never used again. The remaining operations, enter, try_enter,
/// leave, and set_spin_count, are all safe to use.
///
/// Calling get_ref() will return a value that can bypass an initialization check for all
/// operations.
pub struct CriticalStatic {
    // init_spin_count Safety: this is only set during contruction and never modified.
    // Therefore, no concern about thread-safety.
    init_spin_count: Option<u32>,
    init: AtomicUsize,
    // Need UnsafeCell for interior mutability (mutation happens through FFI)
    inner: PoisonableCriticalSection,
}

// Safety note:
// CriticalStaticRef<Init> can be copied freely.
// CriticalStaticRef<Uninit> may not be copied or cloned.
// This allows CriticalStaticRef::<Uninit>::init() to be safe.
#[derive(Copy, Clone)]
pub struct CriticalStaticRef<State>(&'static PoisonableCriticalSection, State);
#[derive(Copy, Clone)]
pub struct Init;
pub struct Uninit;

// Safety: Send and Sync are safe becuase these types work with &'static CRITICAL_SECTION.
unsafe impl Sync for CriticalStatic {}
unsafe impl<State> Send for CriticalStaticRef<State> {}
unsafe impl<State> Sync for CriticalStaticRef<State> {}

impl CriticalStatic {
    /// Creates a new CriticalStatic.
    pub const fn new() -> Self {
        Self {
            init_spin_count: None,
            init: AtomicUsize::new(UNINITIALIZED),
            inner: POISONABLE_ZEROED,
        }
    }
    /// Creates a new CriticalStatic which will be initialized with the provided spin_count.
    pub const fn with_spin_count(spin_count: u32) -> Self {
        Self {
            init_spin_count: Some(spin_count),
            init: AtomicUsize::new(UNINITIALIZED),
            inner: POISONABLE_ZEROED,
        }
    }
    fn init_once(&'static self) {
        struct PoisonCatcher<'a>(&'a AtomicUsize);
        impl Drop for PoisonCatcher<'_> {
            fn drop(&mut self) {
                self.0.store(POISONED, Ordering::Relaxed)
            }
        }
        if INITIALIZED == self.init.load(Ordering::Acquire) {
            return;
        } else if self
            .init
            .compare_exchange(
                UNINITIALIZED,
                INITIALIZING,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_ok()
        {
            let catcher = PoisonCatcher(&self.init);
            if let Some(spin_count) = self.init_spin_count {
                unsafe {
                    init_cs_with_spin_count(self.lpCriticalSection(), spin_count);
                }
            } else {
                unsafe {
                    init_cs(self.lpCriticalSection());
                }
            }
            core::mem::forget(catcher);
            self.init.store(INITIALIZED, Ordering::Release);
            return;
        } else {
            // It won't take long, just spin
            loop {
                let status = self.init.load(Ordering::Acquire);
                if INITIALIZED == status {
                    return;
                } else if POISONED == status {
                    panic!("Critical Section init failed")
                }
            }
        }
    }
    #[allow(non_snake_case)]
    fn lpCriticalSection(&'static self) -> *mut CRITICAL_SECTION {
        self.inner.critical.get()
    }
    /// Enters the Critical Section. This will not deadlock if the
    /// calling thread is already in the Critical Section.
    pub fn enter(&'static self) -> EnteredCritical<'static> {
        self.init_once();
        // Safety: might panic, no return value. Naturally thread-safe.
        unsafe {
            enter_cs(self.lpCriticalSection());
            EnteredCritical::new(&self.inner)
        }
    }
    /// Tries to enter the critical section without blocking. This will
    /// not deadlock if the calling thread is already in the Critical
    /// Section.
    pub fn try_enter(&'static self) -> Option<EnteredCritical<'static>> {
        self.init_once();
        // Safety: returns non-zero if we are in critical section when call returns.
        // Naturally thread-safe.
        unsafe {
            match try_enter_cs(self.lpCriticalSection()) {
                0 => None,
                _ => Some(EnteredCritical::new(&self.inner)),
            }
        }
    }
    /// Sets the spin count of this Critical Section, and returns the
    /// old value
    pub fn set_spin_count(&'static self, spin_count: u32) -> u32 {
        self.init_once();
        // Safety: cannot fail. Returns previous spin_count. Naturally thread-safe.
        unsafe { set_cs_spin_count(self.lpCriticalSection(), spin_count) }
    }
    /// Gets a thin reference to the CriticalStatic, bypassing initialization checks
    /// on future operations. The returned reference is Copy, Send, and Sync.
    pub fn get_ref(&'static self) -> CriticalStaticRef<Init> {
        self.init_once();
        CriticalStaticRef(
            // Safety: we are init and have &'static, so this is fine
            &self.inner,
            Init,
        )
    }
    pub unsafe fn assume_uninit(&'static self) -> CriticalStaticRef<Uninit> {
        CriticalStaticRef(&self.inner, Uninit)
    }
    pub unsafe fn delete(&'static self) -> CriticalStaticRef<Uninit> {
        delete_cs(self.lpCriticalSection());
        self.assume_uninit()
    }
    pub unsafe fn init(&'static self) {
        self.inner.clear_poison_unsynced();
        init_cs(self.lpCriticalSection())
    }
    pub unsafe fn init_with_spin_count(&'static self, spin_count: u32) -> CriticalStaticRef<Init> {
        self.inner.clear_poison_unsynced();
        init_cs_with_spin_count(self.lpCriticalSection(), spin_count);
        self.get_ref()
    }
}

impl<State> CriticalStaticRef<State> {
    #[allow(non_snake_case)]
    fn lpCriticalSection(&self) -> *mut CRITICAL_SECTION {
        self.0.critical.get()
    }
}

impl CriticalStaticRef<Uninit> {
    pub fn init(self) -> CriticalStaticRef<Init> {
        unsafe {
            self.0.clear_poison_unsynced();
            init_cs(self.lpCriticalSection());
        }
        CriticalStaticRef(self.0, Init)
    }
    pub fn init_with_spin_count(self, spin_count: u32) -> CriticalStaticRef<Init> {
        unsafe {
            self.0.clear_poison_unsynced();
            init_cs_with_spin_count(self.lpCriticalSection(), spin_count);
        }
        CriticalStaticRef(self.0, Init)
    }
}

impl CriticalStaticRef<Init> {
    pub fn enter(&self) -> EnteredCritical<'static> {
        // Safety: might panic, no return value. Naturally thread-safe.
        unsafe { enter_cs(self.lpCriticalSection()) }
        EnteredCritical::new(self.0)
    }
    pub fn try_enter(&self) -> Option<EnteredCritical> {
        // Safety: returns non-zero if we are in critical section when call returns.
        // Naturally thread-safe.
        match unsafe { try_enter_cs(self.lpCriticalSection()) } {
            0 => None,
            _ => Some(EnteredCritical::new(self.0)),
        }
    }
    pub fn set_spin_count(&self, spin_count: u32) -> u32 {
        // Safety: cannot fail. Returns previous spin_count. Naturally thread-safe.
        unsafe { set_cs_spin_count(self.lpCriticalSection(), spin_count) }
    }
    pub unsafe fn delete(self) -> CriticalStaticRef<Uninit> {
        delete_cs(self.lpCriticalSection());
        CriticalStaticRef(self.0, Uninit)
    }
}

#[cfg(test)]
mod tests {
    use crate::CriticalStatic;
    use std::thread;

    #[test]
    fn threads_on_the_wall() {
        static mut X: usize = 0;
        static CRITICAL: CriticalStatic = CriticalStatic::new();
        let mut handles = Vec::with_capacity(99);
        for i in 0..99 {
            handles.push(thread::spawn(move || {
                let entered = CRITICAL.enter();
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
        assert!(CRITICAL.enter().is_poisoned());
    }

    #[test]
    fn threads_on_the_wall_ref() {
        static mut X: usize = 0;
        static CRITICAL: CriticalStatic = CriticalStatic::new();
        let crit_ref = CRITICAL.get_ref();
        let mut handles = Vec::with_capacity(99);
        for i in 0..99 {
            handles.push(thread::spawn(move || {
                let entered = crit_ref.enter();
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
        assert!(crit_ref.enter().is_poisoned());
    }
}
