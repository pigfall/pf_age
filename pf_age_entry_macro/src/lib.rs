use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn game_main_wrapper(attr: proc_macro::TokenStream,input: proc_macro::TokenStream)->proc_macro::TokenStream{
    let item_ast = parse_macro_input!(input as ItemFn);
    let main_fn_name = &item_ast.sig.ident;

    let output_tks = quote!{
        #[cfg(not(target_os="android"))]
        compile_error!("only supported android");

        use std::os::raw::c_void;
        pub use pf_age_entry;
        pub use pf_age_entry::log::info;

        #[no_mangle]
        unsafe extern "C" fn ANativeActivity_onCreate(
            activity_raw_ptr: *mut c_void,
            saved_state: *mut c_void,
            safed_stae_size:usize,
            ){
            // { init logger
            pf_age_entry::init_android_logger("pf_age_logger");
            info!(" ANativeActivity_onCreating");
            // }
            pf_age_entry::onCreateANativeActivity(
                activity_raw_ptr as *mut _,
                saved_state,
                safed_stae_size,
                #main_fn_name,
                );
        }
        #item_ast
    };

    // into tokenstream from tokenstremv2
    output_tks.into()
}

