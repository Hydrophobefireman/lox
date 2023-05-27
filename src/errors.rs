pub struct RuntimeError {
    message: String,
}
impl RuntimeError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.into(),
        }
    }
}
