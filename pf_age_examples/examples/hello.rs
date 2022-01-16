//use pf_age_app::App;
#[cfg_attr(
    target_os="android",
    pf_age_entry::game_main_wrapper()
    )]
pub fn main(){
    //App::new().run();
}
