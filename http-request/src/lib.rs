#[cfg(not(feature = "wasm"))]
mod native;

#[cfg(not(feature = "wasm"))]
pub use native::http_request;

#[cfg(feature = "wasm")]
mod wasm;

#[cfg(feature = "wasm")]
pub use wasm::http_request;