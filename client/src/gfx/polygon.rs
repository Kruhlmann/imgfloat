use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::bindings::random;

use super::Size2D;

const POLYGON_MIN_SIZE_PX: f64 = 10.0;
const POLYGON_MAX_SIZE_PX: f64 = 50.0;
const POLYGON_MIN_VERTICES_COUNT: f64 = 3.0;
const POLYGON_MAX_VERTICES_COUNT: f64 = 8.0;

// Example colors
static POLYGON_COLORS: [&str; 5] = ["#FF0000", "#00FF00", "#0000FF", "#FFFF00", "#FF00FF"];

fn rand_range(min: f64, max: f64) -> f64 {
    min + random() * (max - min)
}

pub struct Polygon {
    x: f64,
    y: f64,
    size: f64,
    vx: f64,
    vy: f64,
    rotation: f64,
    rotation_speed: f64,
    vertices: Vec<(f64, f64)>,
    color: String,
}

impl Polygon {
    pub fn new(spawn_inside: bool, canvas_size: &Size2D<f64>) -> Polygon {
        let size = rand_range(POLYGON_MIN_SIZE_PX, POLYGON_MAX_SIZE_PX);
        let color_index = (random() * POLYGON_COLORS.len() as f64) as usize;
        let color = POLYGON_COLORS[color_index.min(POLYGON_COLORS.len() - 1)].to_string();
        let mut polygon = Polygon {
            x: 0.0,
            y: 0.0,
            size,
            vx: 0.0,
            vy: 0.0,
            rotation: random() * 2.0 * std::f64::consts::PI,
            rotation_speed: (random() - 0.5) * 0.05,
            vertices: vec![],
            color,
        };

        polygon.vertices = polygon.generate_vertices();

        if spawn_inside {
            polygon.x = random() * canvas_size.width;
            polygon.y = random() * canvas_size.height;

            let angle = random() * 2.0 * std::f64::consts::PI;
            let speed = random() * 8.0 + 1.0;

            polygon.vx = angle.cos() * speed;
            polygon.vy = angle.sin() * speed;
        } else {
            let side = (random() * 4.0).floor() as i32;
            match side {
                0 => {
                    polygon.x = random() * canvas_size.width;
                    polygon.y = -polygon.size * 2.0;
                    polygon.vx = (random() - 0.5) * 2.0;
                    polygon.vy = random() * 2.0 + 1.0;
                }
                1 => {
                    polygon.x = canvas_size.width + polygon.size * 2.0;
                    polygon.y = random() * canvas_size.height;
                    polygon.vx = -(random() * 2.0 + 1.0);
                    polygon.vy = (random() - 0.5) * 2.0;
                }
                2 => {
                    polygon.x = random() * canvas_size.width;
                    polygon.y = canvas_size.height + polygon.size * 2.0;
                    polygon.vx = (random() - 0.5) * 2.0;
                    polygon.vy = -(random() * 2.0 + 1.0);
                }
                _ => {
                    polygon.x = -polygon.size * 2.0;
                    polygon.y = random() * canvas_size.height;
                    polygon.vx = random() * 2.0 + 1.0;
                    polygon.vy = (random() - 0.5) * 2.0;
                }
            }
        }

        polygon
    }

    pub fn draw(&self, ctx: &CanvasRenderingContext2d) {
        ctx.save();
        ctx.translate(self.x, self.y).unwrap();
        ctx.rotate(self.rotation).unwrap();
        ctx.begin_path();
        ctx.move_to(self.vertices[0].0.into(), self.vertices[0].1.into());
        for vertex in self.vertices.iter() {
            ctx.line_to(vertex.0, vertex.1);
        }
        ctx.close_path();
        ctx.set_fill_style(&JsValue::from(&self.color));
        ctx.fill();
        ctx.restore();
    }

    fn generate_vertices(&self) -> Vec<(f64, f64)> {
        let vertex_count =
            (rand_range(POLYGON_MIN_VERTICES_COUNT, POLYGON_MAX_VERTICES_COUNT + 1.0)) as i32;

        let angle_step = (2.0 * std::f64::consts::PI) / (vertex_count as f64);
        let mut verts = Vec::with_capacity(vertex_count as usize);

        for i in 0..vertex_count {
            let angle = i as f64 * angle_step;
            let radius = self.size * (0.8 + random() * 0.4);
            let vx = angle.cos() * radius;
            let vy = angle.sin() * radius;
            verts.push((vx, vy));
        }

        verts
    }

    pub fn update(&mut self) -> &mut Self {
        self.x += self.vx;
        self.y += self.vy;
        self.rotation += self.rotation_speed;
        self
    }

    pub fn is_off_screen(&self, canvas_size: &Size2D<f64>) -> bool {
        let margin = self.size * 3.0;
        self.x < -margin
            || self.x > canvas_size.width + margin
            || self.y < -margin
            || self.y > canvas_size.height + margin
    }
}
