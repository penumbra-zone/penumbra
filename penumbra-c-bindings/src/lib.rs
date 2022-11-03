// Required because of NCT type size
#![recursion_limit = "256"]

// is exporting add and flip from ..view/c.rs directly possible 
// so `cbindgen --lang c --output include/penumbra_c_bindings.h` generates without below
pub mod view;
