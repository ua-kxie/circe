use crate::ngspice::*;
use crate::PkSpiceManager;
use libc::*;

pub unsafe extern "C" fn cbw_send_char<T>(
    msg: *const c_char,
    id: c_int,
    user: *const c_void,
) -> c_int
where
    T: PkSpiceManager,
{
    unsafe {
        <T as PkSpiceManager>::cb_send_char(
            &mut *(user as *mut T),
            std::ffi::CStr::from_ptr(msg).to_str().unwrap().to_owned(),
            id,
        );
    }
    0
}
pub unsafe extern "C" fn cbw_send_stat<T>(
    msg: *const c_char,
    id: c_int,
    user: *const c_void,
) -> c_int
where
    T: PkSpiceManager,
{
    unsafe {
        <T as PkSpiceManager>::cb_send_stat(
            &mut *(user as *mut T),
            std::ffi::CStr::from_ptr(msg).to_str().unwrap().to_owned(),
            id,
        );
    }
    0
}
pub unsafe extern "C" fn cbw_controlled_exit<T>(
    status: c_int,
    immediate: bool,
    exit_on_quit: bool,
    id: c_int,
    user: *const c_void,
) -> c_int
where
    T: PkSpiceManager,
{
    unsafe {
        <T as PkSpiceManager>::cb_ctrldexit(
            &mut *(user as *mut T),
            status,
            immediate,
            exit_on_quit,
            id,
        );
    }
    0
}
pub unsafe extern "C" fn cbw_send_data<T>(
    pvecvaluesall: *const NgVecvaluesall,
    count: c_int,
    id: c_int,
    user: *const c_void,
) -> c_int
where
    T: PkSpiceManager,
{
    // todo: should be an option to bypass this code if the result is not used
    // create native PkVecvaluesall
    let pkvecinfoall = (*pvecvaluesall).to_pk();

    // call native callback
    <T as PkSpiceManager>::cb_send_data(&mut *(user as *mut T), pkvecinfoall, count, id);
    0
}
pub unsafe extern "C" fn cbw_send_init_data<T>(
    pvecinfoall: *const NgVecinfoall,
    id: c_int,
    user: *const c_void,
) -> c_int
where
    T: PkSpiceManager,
{
    // todo: should be an option to bypass this code if the result is not used
    // create native PkVecInfoall
    let pkvecinfoall = (*pvecinfoall).to_pk();

    // call native callback
    <T as PkSpiceManager>::cb_send_init(&mut *(user as *mut T), pkvecinfoall, id);
    0
}
pub unsafe extern "C" fn cbw_bgthread_running<T>(
    finished: bool,
    id: c_int,
    user: *const c_void,
) -> c_int
where
    T: PkSpiceManager,
{
    unsafe {
        <T as PkSpiceManager>::cb_bgt_state(&mut *(user as *mut T), finished, id);
    }
    0
}
