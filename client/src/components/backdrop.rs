#![allow(non_snake_case)]

use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::{
    bindings::{document, performance_now, request_animation_frame, window, window_size},
    gfx::{Polygon, Size2D},
};

static CSS: Asset = asset!("/assets/css/backdrop.scss");

fn resize_canvas(canvas: &HtmlCanvasElement) -> Size2D<f64> {
    let size = window_size();
    canvas.set_width((size.width as i32).try_into().unwrap());
    canvas.set_height((size.height as i32).try_into().unwrap());
    size
}

pub fn BackDrop() -> Element {
    let mut canvas = use_signal(|| None);
    use_effect(move || {
        if (&canvas.read()).is_none() {
            return;
        }
        let canvas: HtmlCanvasElement = document()
            .get_element_by_id("demo-backdrop")
            .expect("canvas element")
            .dyn_into()
            .unwrap();
        let context: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();
        let window_size = resize_canvas(&canvas);
        let polygons = Rc::new(RefCell::new(vec![]));
        for _ in 0..15 {
            polygons.borrow_mut().push(Polygon::new(true, &window_size));
        }
        let draw_reference = Rc::new(RefCell::new(None));
        let initial_draw_reference = draw_reference.clone();
        let mut time_of_last_frame: f64 = 0.0;
        let mut time_since_last_fps_report = 0.0;
        let mut frames_since_last_fps_report = 0;

        *initial_draw_reference.borrow_mut() = Some(Closure::new(move || {
            request_animation_frame(draw_reference.borrow().as_ref().unwrap());
            let window_size = resize_canvas(&canvas);
            let now = performance_now();
            let delta = now - time_of_last_frame;
            time_of_last_frame = now;
            context.clear_rect(0.0, 0.0, window_size.width, window_size.height);
            for polygon in polygons.borrow_mut().iter_mut() {
                polygon.update().draw(&context);
                if polygon.is_off_screen(&window_size) {
                    *polygon = Polygon::new(false, &window_size);
                }
            }

            if time_since_last_fps_report > 1000.0 {
                tracing::info!(fps = ?frames_since_last_fps_report, "fps");
                time_since_last_fps_report = 0.0;
                frames_since_last_fps_report = 0;
            }
            time_since_last_fps_report += delta;
            frames_since_last_fps_report += 1;
        }));

        request_animation_frame(initial_draw_reference.borrow().as_ref().unwrap());
    });

    rsx! {
        document::Stylesheet { href: CSS }
        section { class: "backdrop",
            canvas {
                id: "demo-backdrop",
                onmounted: move |element| canvas.set(Some(element)),
            }
        }
    }
}
