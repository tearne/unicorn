use std::borrow::Cow;

pub type BoxedError = Box<dyn std::error::Error + Sync + Send>;

#[derive(Debug)]
pub struct AppError(Cow<'static, str>);
impl AppError {
    pub fn boxed(str: &'static str) -> BoxedError {
        Box::new(AppError(str.into()))
    }
}
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl std::error::Error for AppError {}
