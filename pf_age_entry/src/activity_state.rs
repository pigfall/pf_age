use pf_ndk_raw::{ANativeActivity, ANativeWindow,AInputQueue};
use std::ptr;
use lazy_static::lazy_static;
use std::sync::{Condvar,Mutex};
use std::collections::VecDeque;
use pf_age_event::{Event,SystemEvent};
use log::info;
use shrev::{EventChannel};

pub struct ActivityState{
    pub native_activity: *mut ANativeActivity, 
    pub native_window: *mut ANativeWindow,
    pub input_queue: *mut AInputQueue,
    pub updated: bool,
    pub cond_var:Condvar,
    pub mutex: Mutex<bool>,
    pub activity_evs:VecDeque<Event>,
    pub game_event_channel: EventChannel<Event>,
    pub gl_fc_loaded: bool,
}


impl ActivityState {
    /*
    forward_event(&mut self,event :Event)
    update_native_window(&mut self,window: *mut ANativeWindow)
    update_input_queue(&mut self,input_queue: *mut AInputQueue)
     
    */
    pub fn forward_event(&mut self,event :Event){
    }

    pub fn destroy_window(&mut self){
         let mut guard = self.mutex.lock().map_err(|e|{info!("{:?}",e);e}).unwrap();
        self.updated =false;
        self.activity_evs.push_back(Event::SystemEvent(SystemEvent::AndroidNativeWindowDestoryed));
        while !self.updated {
             guard  = self.cond_var.wait(guard).unwrap();
        }
        // NOTE ensure the window change to null after the game app has pre handle the event
        self.native_window = ptr::null_mut();
    }
    pub fn update_native_window(&mut self,window: *mut ANativeWindow){
        // { wait_window_replaced;
         let mut guard = self.mutex.lock().map_err(|e|{info!("{:?}",e);e}).unwrap();
        self.native_window = window;
        self.updated =false;
        self.activity_evs.push_back(Event::SystemEvent(SystemEvent::AndroidNativeWindowCreated));
        while !self.updated {
             guard  = self.cond_var.wait(guard).unwrap();
        }
        // } 
    }

    pub fn update_input_queue(&mut self,input_queue: *mut AInputQueue){
        //todo!("");
    }
}


impl Default for ActivityState{
    fn default()->Self{
        Self{
            native_activity:  ptr::null_mut(),
            native_window:  ptr::null_mut(),
            input_queue:  ptr::null_mut(),
            updated:false,
            cond_var:Condvar::new(),
            mutex: Mutex::new(false),
            activity_evs:VecDeque::with_capacity(200),
            game_event_channel: EventChannel::new(),
            gl_fc_loaded:false,
        }
    }
}




pub static mut activity_state: Option<ActivityState>=None;

pub fn get_act_state()->&'static mut ActivityState{
    return  unsafe{activity_state.as_mut().unwrap()};
}
