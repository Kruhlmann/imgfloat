#[derive(Debug, PartialEq)]
pub struct Percentage(pub f32);

impl Percentage {
    pub fn new<T>(value: T) -> Result<Percentage, f32>
    where
        T: Into<f32> + std::fmt::Debug,
    {
        let value: f32 = value.into();
        if value > 100_f32 {
            tracing::error!(?value, "invalid percentage value");
            return Err(value);
        };
        if value < 0_f32 {
            tracing::error!(?value, "invalid percentage value");
            return Err(value);
        }
        Ok(Self(value))
    }
}

impl Into<f32> for Percentage {
    fn into(self) -> f32 {
        self.0
    }
}
