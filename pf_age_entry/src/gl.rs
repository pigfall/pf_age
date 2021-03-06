pub struct GLIns {
    pub egl_ins:  egl::DynamicInstance<egl::EGL1_4>,
    pub display:egl::Display,
    pub config: egl::Config,
    pub ctx:  egl::Context,
    pub surface: Option<egl::Surface>,
    pub gl_fcs: Option<glow::Context>,
}
