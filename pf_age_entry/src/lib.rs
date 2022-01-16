use pf_ndk_raw::{ANativeActivity, ANativeWindow};
use std::os::raw::{c_void,c_int};
use log::info;
use std::ptr::NonNull;
use std::thread;

pub use pf_age_entry_macro::*;

pub use android_logger;
pub use log;

mod activity_state;
use activity_state::ActivityState;



pub fn init_android_logger(tag: &str){
    android_logger::init_once(
        android_logger::Config::default().
        with_min_level(log::Level::Trace). // NOTE: must need min level
        with_tag(tag),
        );
}

static mut callback_counter:i32 = 0;

pub unsafe fn onCreateANativeActivity(
    activity_raw_pointer: *mut ANativeActivity,
    saved_state: *mut c_void,
    saved_state_size: usize,
    ){
    // { fill callbacks
    let mut activity_nonnull_ptr = NonNull::new(activity_raw_pointer).ok_or_else(||{let msg ="unexpted activity is nil"; info!("{:?}",msg);msg}).unwrap();
    let mut callbacks = activity_nonnull_ptr.as_mut().callbacks.as_mut().ok_or_else(||{let msg = "Unexpted: activity's callback is nil";info!("{:?}",msg);msg}).unwrap();
    callbacks.onStart= Some(on_start);
    callbacks.onNativeWindowCreated  = Some(on_native_window_created);
    callbacks.onNativeWindowDestroyed = Some(on_native_window_destroyed);
    callbacks.onWindowFocusChanged =Some(on_native_window_focus_changed);
    info!("✅  callback register success");
    // }
    
    let mut state = activity_state::ActivityState::default();
    state.native_activity = activity_raw_pointer;
    activity_state::activity_state = Some(state);

    thread::spawn(||{
        let mut state = unsafe {
            activity_state::get_act_state()
        };
        while true {
            if let Some(ev) = state.poll_event(){
                state.updated=true;
                state.cond_var.notify_all();
                info!("event : {:?}",ev);
            };
        };
    });
}


unsafe extern "C" fn on_start (activity_raw_ptr: *mut ANativeActivity){
    info!("{:?} on_start function called",callback_counter+=1);
}

unsafe extern "C" fn on_native_window_created(native_activity_raw_ptr: *mut ANativeActivity,native_window_raw_ptr: *mut ANativeWindow){
    info!("{:?} on_native_window_created function called",callback_counter+=1);
    let mut state = activity_state::get_act_state();
    state.update_native_window(native_window_raw_ptr);
}

unsafe extern "C" fn on_native_window_destroyed(native_activity_raw_ptr: *mut ANativeActivity,native_window_raw_ptr: *mut ANativeWindow){
    info!("{:?} on_native_window_destroyed function called",callback_counter+=1);
}

unsafe extern "C" fn on_native_window_focus_changed(native_activity_raw_ptr: *mut ANativeActivity,has_focused: c_int){
    info!("{:?} on_native_window_focus_changed function called",callback_counter+1);
}
