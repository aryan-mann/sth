// TODO: Expand error types instead of putting everything into unknown
pub enum SvixError {
    Unknown { message: String }
}

impl Into<String> for SvixError {
    fn into(self) -> String {
        match self {
            SvixError::Unknown { message } => message,
        }
    }
}

impl SvixError {
    pub fn new(msg: &str) -> SvixError {
        Self::Unknown { message: String::from(msg) }
    }
}

