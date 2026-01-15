//! WebAssembly bindings for World Cup simulation.
//!
//! This crate exposes the simulation engine to JavaScript via wasm-bindgen.

use wasm_bindgen::prelude::*;

mod api;

pub use api::*;

/// Initialize panic hook for better error messages in browser console.
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
