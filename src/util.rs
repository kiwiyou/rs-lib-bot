pub struct TextBuilder {
    buffer: String,
}

impl TextBuilder {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    pub fn text(mut self, prefix: &str, text: impl AsRef<str>, suffix: &str) -> Self {
        self.buffer.push_str(prefix);
        self.buffer.push_str(text.as_ref());
        self.buffer.push_str(suffix);
        self
    }

    pub fn text_opt(self, prefix: &str, text: &Option<impl AsRef<str>>, suffix: &str) -> Self {
        if let Some(inner) = text {
            self.text(prefix, inner, suffix)
        } else {
            self
        }
    }

    pub fn build(self) -> String {
        self.buffer
    }
}
