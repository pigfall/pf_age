use pf_ndk_raw::{ANativeActivity, ANativeWindow,AInputQueue,ALooper_prepare,ALOOPER_PREPARE_ALLOW_NON_CALLBACKS,AInputQueue_attachLooper,AInputQueue_getEvent,AInputEvent,AInputQueue_detachLooper,AInputQueue_finishEvent};
use std::ffi::{CStr, CString};
use glow::HasContext;
use std::os::raw::{c_void,c_int};
use std::io::{BufRead, BufReader};
use log::{info,error};
use std::fs::File;
use std::os::unix::io::FromRawFd;
use std::os::unix::prelude::RawFd;
use std::ptr::NonNull;
use pf_age_event::{Event,SystemEvent};
use std::thread;

mod support;

pub use pf_age_entry_macro::*;

pub use android_logger;
pub use log;
pub use glow;

mod gl;
pub mod prelude;

pub mod activity_state;
use activity_state::ActivityState;

pub use shrev::ReaderId;

mod render;



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
    game_update: fn(game_ev_reader: &mut ReaderId<Event>),
    ){

    let mut logpipe: [RawFd; 2] = Default::default();
    libc::pipe(logpipe.as_mut_ptr());
    libc::dup2(logpipe[1], libc::STDOUT_FILENO);
    libc::dup2(logpipe[1], libc::STDERR_FILENO);
    thread::spawn(move || {
        // let tag = CStr::from_bytes_with_nul(b"pf_age_logger\0").unwrap();

        let file = File::from_raw_fd(logpipe[0]);
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();
        loop {
            buffer.clear();
            if let Ok(len) = reader.read_line(&mut buffer) {
                if len == 0 {
                    break;
                } else if let Ok(msg) = CString::new(buffer.clone()) {
                    error!("{:?}",msg);
                    // android_logger(Level::Info, tag, &msg);
                }
            }
        }
    });

    // { fill callbacks
    let mut activity_nonnull_ptr = NonNull::new(activity_raw_pointer).ok_or_else(||{let msg ="unexpted activity is nil"; info!("{:?}",msg);msg}).unwrap();
    let mut callbacks = activity_nonnull_ptr.as_mut().callbacks.as_mut().ok_or_else(||{let msg = "Unexpted: activity's callback is nil";info!("{:?}",msg);msg}).unwrap();
    callbacks.onStart= Some(on_start);
    callbacks.onNativeWindowCreated  = Some(on_native_window_created);
    callbacks.onNativeWindowDestroyed = Some(on_native_window_destroyed);
    callbacks.onWindowFocusChanged =Some(on_native_window_focus_changed);
    callbacks.onInputQueueCreated = Some(on_input_queue_created);
    callbacks.onInputQueueDestroyed = Some(on_input_queue_destroyed);
    info!("✅  callback register success");
    // }
    
    let mut state = activity_state::ActivityState::default();
    state.native_activity = activity_raw_pointer;
    let mut game_ev_reader_id = state.game_event_channel.register_reader();

    activity_state::activity_state = Some(state);


    //init_egl();
    //info!("✅  egl inited");


    thread::spawn(move||{
        let native_looper = ALooper_prepare(ALOOPER_PREPARE_ALLOW_NON_CALLBACKS as _ );
        if native_looper.is_null(){
            error!("❌ ALooper_prepare failed");
            panic!("❌ ALooper_prepare failed");
        }
        activity_state::get_act_state().native_looper =  native_looper;
        loop{
            pre_handle_evs();
            //game_app_update(&mut game_ev_reader_id);
            game_update(&mut game_ev_reader_id);
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
    let mut state = activity_state::get_act_state();
    state.destroy_window();
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

unsafe extern "C" fn on_input_queue_destroyed(
    activity: *mut ANativeActivity,
    queue: *mut AInputQueue,
) {
    callback_counter+=1;
    info!("{:?} on_input_queue_destroyed",callback_counter);
    let mut state = activity_state::get_act_state();
    state.input_queue_destroyed();
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
                pre_handle_native_activity_ev(&ev);
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

    if !activity_state.input_queue.is_null(){
        loop{
            let mut out_event = std::ptr::null_mut();
            unsafe{
                if AInputQueue_getEvent(activity_state.input_queue,&mut out_event)<0{
                    break;
                }else{
                    info!("get input event {:?}",out_event);
                    AInputQueue_finishEvent(activity_state.input_queue,out_event,1);
                };
            };
        };
    };
    //write_to_event_channel(native_activity_evs,input_evs);
    // }
}

fn write_to_event_channel(ev:Event){
    let activity_state = activity_state::get_act_state();
    activity_state.game_event_channel.single_write(ev);
}

fn game_app_update(game_ev_reader: &mut shrev::ReaderId<Event>){
    let activity_state = activity_state::get_act_state();
    for ev in activity_state.game_event_channel.read(game_ev_reader){
        info!("✅     read from game ev channel {:?}",ev);
    }
    let gl_wrapper = activity_state.gl.as_ref().unwrap();
    if let Some(gl_fcs) = gl_wrapper.gl_fcs.as_ref(){
        if let Some(surface) = gl_wrapper.surface{
            unsafe{
                gl_fcs.clear_color(0.1,0.2,0.3,1.0);
                gl_fcs.clear(glow::COLOR_BUFFER_BIT);
                gl_wrapper.egl_ins.swap_buffers(gl_wrapper.display,surface);
            };
        };
    };
}


fn pre_handle_native_activity_ev(ev :&Event){
    match ev {
        Event::SystemEvent(systemEvent)=>{
            match systemEvent{
                SystemEvent::AndroidNativeWindowCreated=>{
                    let act_state = activity_state::get_act_state();
                    let window_ptr = act_state.native_window;
                    if !act_state.egl_inited{
                        info!("⌛ initing egl");
                        init_egl();
                        act_state.egl_inited = true;
                    }
                    let mut gl_wrapper = &mut act_state.gl.as_mut().unwrap();
                    // { create render context
                    // }

                    // { loaded  gl function if not loaded
                    //
                    //

                    info!("⌛ Creating window surface");
                    let surface = unsafe {
                        gl_wrapper.egl_ins.create_window_surface(gl_wrapper.display,gl_wrapper.config,window_ptr as egl::NativeWindowType,None).
                            map_err(
                                |e|{
                                    info!("❌ Failed to create window surface {:?}",e);
                                    e
                                }
                                ).unwrap()
                    }; 
                    info!("✅  Created window surface");
                    gl_wrapper.surface=Some(surface);
                    // >>

                    // <<  Attach an EGL rendering context to EGL surfaces.
                    info!("⌛ Attach an EGL rendering context to EGL surfaces");
                    gl_wrapper.egl_ins.make_current(gl_wrapper.display,Some(surface),Some(surface),Some(gl_wrapper.ctx)).
                        map_err(
                            |e|{
                                info!("❌ Failed to attach egl rendering context to egl surfaces");
                                e
                            }
                            ).unwrap();
                    info!("✅ Attached an EGL rendering context to EGL surfaces");
                    if !act_state.gl_fc_loaded{
                        support::load(
                             |name|{
                                    info!("⌛ Loading {:?}",name);
                                    gl_wrapper.egl_ins.get_proc_address(name).
                                        map_or(std::ptr::null(),|ptr|{
                                            info!("✅  Loaded {:?} {:?}",name,ptr);
                                            ptr as *const _
                                        })

                                }   
                            );
                        //panic!("dedbug");
                        
                        info!("⌛  Loading gl functions");
                        let gl_fcs = unsafe {
                            //glow::Context::from_loader_function(
                            //    |name|{
                            //        info!("⌛ Loading {:?}",name);
                            //        gl_wrapper.egl_ins.get_proc_address(name).
                            //            map_or(std::ptr::null(),|ptr|{
                            //                info!("✅  Loaded {:?} {:?}",name,ptr);
                            //                ptr as *const _
                            //            })

                            //    }
                            //        
                            //    )


                            glow::Context::from_loader_function_with_version_parse(
                                |version_str|{
                                    // TODO
                                    info!("gl version {:?}",version_str);
                                    Ok(
                                        glow::Version {
                                            major: 1,
                                            minor: 0,
                                            is_embedded: true,
                                            revision: None,
                                            vendor_info: "tzz".to_string(),
                                        }

                                      )
                                }
                                ,
                                |name|{
                                    info!("⌛ Loading {:?}",name);
                                    gl_wrapper.egl_ins.get_proc_address(name).
                                        map_or(std::ptr::null(),|ptr|{
                                            info!("✅  Loaded {:?} {:?}",name,ptr);
                                            ptr as *const _
                                        })
                                }).map_err(
                                    |e|{
                                        info!("❌ {:?}",e);
                                        e
                                    }
                                    ).unwrap()
                        };
                        gl_wrapper.gl_fcs = Some(gl_fcs);

                        act_state.gl_fc_loaded = true;

                    }
                    // }
                },
                SystemEvent::AndroidNativeWindowDestoryed=>{
                    let act_state = activity_state::get_act_state();
                    let gl_wrapper = &act_state.gl.as_ref().unwrap();
                    match gl_wrapper.surface{
                        Some(surface)=>{
                            info!("⌛ destroying surface");
                            gl_wrapper.egl_ins.destroy_surface(gl_wrapper.display,surface);
                            info!("✅  destroyed surface");
                        },
                        _=>{},
                    }
                }
                SystemEvent::AndroidNativeInputQueueCreated=>{
                    let act_state = activity_state::get_act_state();
                    let input_queue =act_state.input_queue; 
                    info!("⌛  Doing AInputQueue_attachLooper");
                    unsafe{
                        AInputQueue_attachLooper(input_queue,act_state.native_looper, NDK_GLUE_LOOPER_INPUT_QUEUE_IDENT,None,std::ptr::null_mut());
                    };
                    info!("✅  AInputQueue_attachLooper Done");
                }
                SystemEvent::AndroidNativeInputQueueDestroyed=>{
                    let act_state = activity_state::get_act_state();
                    let input_queue =act_state.input_queue; 
                    info!("⌛  Doing AInputQueue_detachLooper");
                    unsafe{
                        AInputQueue_detachLooper(input_queue);
                    };
                    info!("✅  AInputQueue_detachLooper Done");
                }
                _=>{},
            }
        }
        _ =>{},
    }
}

pub const NDK_GLUE_LOOPER_INPUT_QUEUE_IDENT: i32 = 1;


fn init_egl(){
    info!("⌛ Loading egl functions");
    let  egl_ins = unsafe {
        egl::DynamicInstance::<egl::EGL1_4>::load_required().map_err(|e|{info!("✅ Failed to load egl functions {:?}",e);e}).unwrap()
    };
    info!("⌛ Getting default display");
    let display = egl_ins.get_display(egl::DEFAULT_DISPLAY).ok_or_else(||{let msg="❌ noget default display";info!("{:?}",msg);msg}).unwrap();
    info!("✅ Getted default display");


    info!("⌛  Initing display");
    egl_ins.initialize(display).map_err(|e|{
        info!("❌ Failed to init display {:?}",e);
        e
    }).unwrap();
    info!("✅ Inited display");


    info!("⌛ Choose config which matched the attributes we wanted");
    let attributes:Vec<egl::Int> = [egl::RED_SIZE, 8, egl::GREEN_SIZE, 8, egl::BLUE_SIZE, 8, egl::NONE].to_vec();
    let config :egl::Config = egl_ins.choose_first_config(display,&attributes).
        map_err(|e|{
            info!("❌ Failed to choose first config {:?}",e);
            e
        }).unwrap().
    ok_or_else(||{let msg = "❌ No matched config ";info!("{:?}",msg);msg}).unwrap();
    info!("✅ Config choosed");
    // >>

    let context_attributes = [
		egl::CONTEXT_MAJOR_VERSION, 2,
		egl::CONTEXT_MINOR_VERSION, 0,
        egl::NONE,
	];

    // << create_context
    info!("⌛ Creating context");
    let ctx = egl_ins.create_context(display,config,None,Some(&context_attributes)).map_err(
        |e|{
            info!("❌ Failed to create context {:?}",e);
            e
        }
        ).unwrap();
    info!("✅ Created context");



    let gl_ins = gl::GLIns{
        display:display,
        ctx:ctx,
        config:config,
        egl_ins:egl_ins,
        surface:None,
        gl_fcs:None,
    };

    activity_state::get_act_state().gl = Some(gl_ins);
}
