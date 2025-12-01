pub trait Traceable {
    fn trace_err(self, msg: &str) -> Self;
}

impl<T, E> Traceable for Result<T, E>
where
    E: std::fmt::Debug,
{
    fn trace_err(self, msg: &str) -> Self {
        if let Err(ref error) = self {
            tracing::error!(?error, msg);
        }
        self
    }
}
