#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub is_local: bool,
}

impl Symbol {
    pub fn new(name: String) -> Self {
        Self {
            name,
            is_local: false,
        }
    }
}
