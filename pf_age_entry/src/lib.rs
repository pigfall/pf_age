use pf_ndk_raw::ANativeActivity;

pub use pf_age_entry_macro::*;

pub use android_logger;
pub use log;

pub fn init_android_logger(tag: &str){
    android_logger::init_once(
        android_logger::Config::default().
        with_min_level(log::Level::Trace). // NOTE: must need min level
        with_tag(tag),
        );

}
