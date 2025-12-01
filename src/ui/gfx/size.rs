pub struct Size2D<T> {
    pub width: T,
    pub height: T,
}

impl Size2D<f64> {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}
