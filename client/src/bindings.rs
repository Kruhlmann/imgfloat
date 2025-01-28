use wasm_bindgen::prelude::*;

use crate::gfx::Size2D;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Math)]
    pub fn random() -> f64;
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("window")
}

pub fn document() -> web_sys::HtmlDocument {
    window()
        .document()
        .expect("no document")
        .dyn_into()
        .unwrap()
}

pub fn performance_now() -> f64 {
    window().performance().expect("performance").now()
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("register requestAnimationFrame");
}

pub fn window_size() -> Size2D<f64> {
    Size2D::new(
        window().inner_width().unwrap().as_f64().unwrap(),
        window().inner_height().unwrap().as_f64().unwrap(),
    )
}
