#[derive(Debug)]
pub struct Error(u64, String);

impl Error {
    pub(crate) fn new(line: u64, msg: impl Into<String>) -> Self {
        Self(line, msg.into())
    }

    /// Returns the input line where html5ever reported this parse error.
    pub fn line(&self) -> u64 {
        self.0
    }

    /// Returns html5ever's parse error message.
    pub fn message(&self) -> &str {
        &self.1
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sxd_html error at line {} : {}", self.0, self.1)
    }
}

impl std::error::Error for Error {}
