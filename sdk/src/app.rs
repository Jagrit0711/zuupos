//! App lifecycle and registration.
pub struct App {
    pub name: String,
    pub version: String,
}

impl App {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
        }
    }
}
