//use pf_age_app::App;
use pf_age_entry::prelude::*;
use glow::HasContext;
use pf_age_entry::glow as glow;
#[cfg_attr(
    target_os="android",
    pf_age_entry::game_main_wrapper()
    )]
pub fn game_app_update(game_ev_reader: &mut pf_age_entry::ReaderId<pf_age_event::Event>){
    let activity_state = pf_age_entry::activity_state::get_act_state();
    for ev in activity_state.game_event_channel.read(game_ev_reader){
        info!("✅     read from game ev channel {:?}",ev);
    }
    let gl_wrapper = activity_state.gl.as_ref().unwrap();
    if let Some(gl_fcs) = gl_wrapper.gl_fcs.as_ref(){
        if let Some(surface) = gl_wrapper.surface{
            unsafe{
                gl_fcs.clear_color(1.0,0.2,0.3,1.0);
                gl_fcs.clear(glow::COLOR_BUFFER_BIT);
                gl_wrapper.egl_ins.swap_buffers(gl_wrapper.display,surface);
            };
        };
    };
}
