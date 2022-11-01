// Required because of NCT type size
#![recursion_limit = "256"]

// is exporting add and flip from ..view/c.rs directly possible 
// so `cbindgen --lang c --output include/penumbra_c_bindings.h` generates without below
pub use penumbra_view::c::{add, flip};

// testing works but this is bad
// could also be in a view.rs ig then pub mod view
#[no_mangle]
pub extern "C" fn view_add(a: i32, b: i32) -> i32 {
    add(a, b)
}

#[no_mangle]
pub extern "C" fn view_flip(a: bool) -> bool {
    flip(a)
}