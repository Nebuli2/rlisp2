#[cfg(not(feature = "wasm"))]
mod app;

#[cfg(not(feature = "wasm"))]
mod repl;

#[cfg(not(feature = "wasm"))]
fn main() {
    app::run();
}

#[cfg(feature = "wasm")]
fn main() {
    println!("Hello, world!");
}
