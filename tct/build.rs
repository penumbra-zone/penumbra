fn main() {
    // Inform cargo about the resources that are baked into the binary
    let resources = [
        "examples/key-control.js",
        "src/live/view/index.html",
        "src/live/view/index.js",
        "src/live/view/reset.css",
        "src/live/view/style.css",
        "node_modules/d3/dist/d3.min.js",
        "node_modules/@hpcc-js/wasm/dist/index.min.js",
        "node_modules/@hpcc-js/wasm/dist/graphvizlib.wasm",
        "node_modules/d3-graphviz/build/d3-graphviz.js",
        "node_modules/d3/LICENSE",
        "node_modules/@hpcc-js/wasm/LICENSE",
        "node_modules/d3-graphviz/LICENSE",
    ];
    for file in resources {
        println!("cargo:rerun-if-changed={file}");
    }
}
