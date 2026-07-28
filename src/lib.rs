mod app;
mod camera;
mod egui_renderer;
mod init_shape;
mod matrix4;
mod particle_chunk;
mod particle_lifetime;
mod particle_vertex;
mod quaternion;
mod renderer;
pub mod run;
mod simulation_parameters;
mod state;
mod texture;
mod vector;
mod view_proj_uniforms;

#[cfg(target_arch = "wasm32")]
use crate::run::run;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();

    Ok(())
}
