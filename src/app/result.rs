use std::fmt::Display;

pub(crate) type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub(crate) struct AppError {
    description: String,
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl AppError {
    pub(crate) fn new(description: String) -> Self {
        Self {
            description: description.to_string()
        }
    }
}
