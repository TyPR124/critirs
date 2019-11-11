use winapi::{
    shared::minwindef::DWORD,
    um::minwinbase::LPCRITICAL_SECTION,
};

#[link(name="wrapper", kind="static")]
extern "C" {
    fn _c_init_cs(lpCriticalSection: LPCRITICAL_SECTION) -> DWORD;
    fn _c_init_cs_with_spin_count(lpCriticalSection: LPCRITICAL_SECTION, spin_count: DWORD) -> DWORD;
    fn _c_enter_cs(lpCriticalSection: LPCRITICAL_SECTION) -> DWORD;
    fn _c_try_enter_cs(lpCriticalSection: LPCRITICAL_SECTION) -> DWORD;
    fn _c_leave_cs(lpCriticalSection: LPCRITICAL_SECTION);
    fn _c_delete_cs(lpCriticalSection: LPCRITICAL_SECTION);
    fn _c_set_cs_spin_count(lpCriticalSection: LPCRITICAL_SECTION, spin_count: DWORD) -> DWORD;
}

#[allow(non_snake_case)]
pub unsafe fn init_cs(lpCriticalSection: LPCRITICAL_SECTION) {
    match _c_init_cs(lpCriticalSection) {
        0 => panic!("Failed to initialize critical section"),
        _ => return
    }
}
#[allow(non_snake_case)]
pub unsafe fn init_cs_with_spin_count(lpCriticalSection: LPCRITICAL_SECTION, spin_count: DWORD) {
    match _c_init_cs_with_spin_count(lpCriticalSection, spin_count) {
        0 => panic!("Failed to initialize critical section"),
        _ => return
    }
}
#[allow(non_snake_case)]
pub unsafe fn enter_cs(lpCriticalSection: LPCRITICAL_SECTION) {
    match _c_enter_cs(lpCriticalSection) {
        0 => panic!("Failed to initialize critical section"),
        _ => return
    }
}
#[allow(non_snake_case)]
pub unsafe fn try_enter_cs(lpCriticalSection: LPCRITICAL_SECTION) -> DWORD {
    _c_try_enter_cs(lpCriticalSection)
}
#[allow(non_snake_case)]
pub unsafe fn leave_cs(lpCriticalSection: LPCRITICAL_SECTION) {
    _c_leave_cs(lpCriticalSection)
}
#[allow(non_snake_case)]
pub unsafe fn delete_cs(lpCriticalSection: LPCRITICAL_SECTION) {
    _c_delete_cs(lpCriticalSection)
}
#[allow(non_snake_case)]
pub unsafe fn set_cs_spin_count(lpCriticalSection: LPCRITICAL_SECTION, spin_count: DWORD) -> DWORD {
    _c_set_cs_spin_count(lpCriticalSection, spin_count)
}