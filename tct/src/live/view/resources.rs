// This is a modified variant of the `flate` macro from the `include_flate` crate, which makes a
// `Bytes` value, so that we can avoid expensive cloning of large strings.
macro_rules! flate_bytes {
($(#[$meta:meta])*
    $(pub $(($($vis:tt)+))?)? static $name:ident from $path:literal) => {
    ::include_flate::lazy_static! {
        $(#[$meta])*
        $(pub $(($($vis)+))?)? static ref $name: ::bytes::Bytes = ::include_flate::decode_string(::include_flate::codegen::deflate_utf8_file!($path)).into();
    }
};
}

// Embed compressed index page
flate_bytes!(pub static INDEX from "src/live/view/index.html");

// Embed compressed source for the relevant javascript libraries
flate_bytes!(pub static D3_JS from "node_modules/d3/dist/d3.min.js");
flate_bytes!(pub static GRAPHVIZ_JS from "node_modules/@hpcc-js/wasm/dist/index.min.js");
flate_bytes!(pub static D3_GRAPHVIZ_JS from "node_modules/d3-graphviz/build/d3-graphviz.js");

// Embed compressed license files for the relevant javascript libraries
flate_bytes!(pub static D3_JS_LICENSE from "node_modules/d3/LICENSE");
flate_bytes!(pub static GRAPHVIZ_JS_LICENSE from "node_modules/@hpcc-js/wasm/LICENSE");
flate_bytes!(pub static D3_GRAPHVIZ_JS_LICENSE from "node_modules/d3-graphviz/LICENSE");
