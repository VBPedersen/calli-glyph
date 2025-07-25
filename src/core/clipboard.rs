#[derive(Debug, Default)]
pub struct Clipboard {
    pub copied_text: Vec<String>,
}

impl Clipboard {
    pub fn new() -> Self {
        Self {
            copied_text: vec![],
        }
    }

    pub fn copy(&mut self, text: &[String]) {
        self.copied_text = text.to_owned();
    }

    pub fn paste(&self) -> Vec<String> {
        self.copied_text.clone()
    }
}