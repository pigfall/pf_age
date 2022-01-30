use pf_ndk_raw::{ANativeActivity, ANativeWindow,AInputQueue};
use std::os::raw::{c_void,c_int};
use log::info;
use std::ptr::NonNull;
use pf_age_event::{Event,SystemEvent};
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
    callbacks.onInputQueueCreated = Some(on_input_queue_created);
    //callbacks.onInputQueueDestroyed = Some(on_input_queue_destroyed);
    info!("✅  callback register success");
    // }
    
    let mut state = activity_state::ActivityState::default();
    state.native_activity = activity_raw_pointer;
    activity_state::activity_state = Some(state);

    thread::spawn(||{
        loop{
            pre_handle_evs();
            //game_app_update()
        }
    });

    //thread::spawn(||{
    //    let mut state = unsafe {
    //        activity_state::get_act_state()
    //    };
    //    while true {
    //        if let Some(ev) = state.poll_event(){
    //            state.updated=true;
    //            state.cond_var.notify_all();
    //            info!("event : {:?}",ev);
    //        };
    //    };
    //});
}


unsafe extern "C" fn on_start (activity_raw_ptr: *mut ANativeActivity){
    callback_counter+=1;
    info!("{:?} on_start function called",callback_counter);
}

unsafe extern "C" fn on_native_window_created(native_activity_raw_ptr: *mut ANativeActivity,native_window_raw_ptr: *mut ANativeWindow){
    callback_counter+=1;
    info!("{:?} on_native_window_created function called",callback_counter);
    let mut state = activity_state::get_act_state();
    state.update_native_window(native_window_raw_ptr);
}

unsafe extern "C" fn on_native_window_destroyed(native_activity_raw_ptr: *mut ANativeActivity,native_window_raw_ptr: *mut ANativeWindow){
    callback_counter+=1;
    info!("{:?} on_native_window_destroyed function called",callback_counter);
}

unsafe extern "C" fn on_native_window_focus_changed(native_activity_raw_ptr: *mut ANativeActivity,has_focused: c_int){
    callback_counter+=1;
    info!("{:?} on_native_window_focus_changed function called",callback_counter);
}

unsafe extern "C" fn on_input_queue_created(
    activity: *mut ANativeActivity,
    queue: *mut AInputQueue,
) {
    callback_counter+=1;
    info!("{:?} on_input_queue_created",callback_counter);
    let mut state = activity_state::get_act_state();
    state.update_input_queue(queue);
}

fn pre_handle_evs(){
    // { summary
    //   1. poll all activity events , and pre handle it then write to Game EventChannel
    //   2. poll all input events then write to GameEventChannel
    // }

    //info!("pre_handle_evs");
    
    let activity_state = activity_state::get_act_state();

    // lock
    activity_state.mutex.lock();

    // { 1. poll poll_nativie_activity_event();
    loop{
        match activity_state.activity_evs.pop_front(){
            None=>break,
            Some(ev)=>{
                // {{ TODO pre handle  :eg udpate window
                info!("handle activity ev {:?}",ev);
                activity_state.updated = true;

                activity_state.cond_var.notify_all();
                // }}
                write_to_event_channel(ev);
            },
        }
    }
    // }
    
    // { 2. poll input events then write to GameEventChannel
    //let input_evs = poll_input_evs();
    //write_to_event_channel(native_activity_evs,input_evs);
    // }
}

fn write_to_event_channel(ev:Event){

}
