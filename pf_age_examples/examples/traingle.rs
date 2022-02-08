use pf_age_entry::glow as glow;
use pf_age_entry::prelude::*;
use pf_age_entry::log::{error};
use glow::HasContext;
use glow::ARRAY_BUFFER;

#[cfg_attr(
    target_os="android",
    pf_age_entry::game_main_wrapper()
    )]
pub fn game_app_update(game_ev_reader: &mut pf_age_entry::ReaderId<pf_age_event::Event>){
    let activity_state = pf_age_entry::activity_state::get_act_state();

    let vertex_shader_str = "#version 330 core
    layout (location = 0) in vec3 aPos;

    void main()
    {
        gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }";

    let frag_shader_str = "
            #version 330 core
            out vec4 FragColor;

            void main()
            {
                FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
            }
    ";

    let gl_wrapper = activity_state.gl.as_ref().unwrap();
    if let Some(gl_fcs) = gl_wrapper.gl_fcs.as_ref(){
        unsafe{
            let vertex_shader =gl_fcs.create_shader(glow::VERTEX_SHADER).unwrap();
            gl_fcs.shader_source(vertex_shader,vertex_shader_str);
            if !gl_fcs.get_shader_compile_status(vertex_shader){
                error!("❌ vertex shader compile error");
                let info = gl_fcs.get_shader_info_log(vertex_shader);
                panic!("❌ shader info {:?}",info);
            }
            let frag_shader = gl_fcs.create_shader(glow:: FRAGMENT_SHADER).unwrap();
            if !gl_fcs.get_shader_compile_status(frag_shader){
                error!("❌ frag shader compile error");
                let info = gl_fcs.get_shader_info_log(frag_shader);
                panic!("❌ shader info {:?}",info);
            }
            let shader_program = gl_fcs.create_program().unwrap();
            gl_fcs.attach_shader(shader_program,vertex_shader);
            gl_fcs.attach_shader(shader_program,frag_shader);
            gl_fcs.link_program(shader_program);
            if !gl_fcs.get_program_link_status(shader_program){
                error!("❌ link program failed");
                let info = gl_fcs. get_program_info_log(shader_program);
                panic!("❌ link program fialed {:?}",info);
            }
            gl_fcs.use_program(Some(shader_program));

            gl_fcs.delete_shader(vertex_shader);
            gl_fcs.delete_shader(frag_shader);

            let activity_state = pf_age_entry::activity_state::get_act_state();
            for ev in activity_state.game_event_channel.read(game_ev_reader){
                info!("✅     read from game ev channel {:?}",ev);
            }
            let vertices = [9f32,-0.5, -0.5, 0.0,
            0.5, -0.5, 0.0,
            0.0,  0.5, 0.0];
            let mut bytes = Vec::<u8>::with_capacity(vertices.len() * 4);
            for vt in vertices.iter() {
                bytes.extend_from_slice(&vt.to_le_bytes());
            }
            let buffer = gl_fcs.create_buffer().unwrap();

            gl_fcs.bind_buffer(ARRAY_BUFFER,Some(buffer));
            gl_fcs.buffer_data_u8_slice(
                ARRAY_BUFFER,
                &bytes,
                glow::STATIC_DRAW,
                );

            gl_fcs.delete_buffer(buffer);
            gl_fcs.delete_program(shader_program);
        }
    }
}

