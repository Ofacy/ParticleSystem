
mod state;
mod app;
mod particle_lifetime;
mod particle_vertex;
mod particle_chunk;
mod init_shape;
mod view_proj_uniforms;
mod simulation_parameters;
mod camera;
mod matrix4;
mod vector;
mod quaternion;
mod texture;
mod renderer;
mod egui_renderer;
mod run;

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
