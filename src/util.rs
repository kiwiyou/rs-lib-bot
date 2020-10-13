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

const MARKDOWN_ESCAPES: [char; 18] = [
    '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!',
];

pub fn escape_markdown(text: &str) -> String {
    let mut buffer = String::with_capacity(text.len());
    for letter in text.chars() {
        if MARKDOWN_ESCAPES.contains(&letter) {
            buffer.push('\\');
        }
        buffer.push(letter);
    }
    buffer
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn text_builder_works() {
        let text = TextBuilder::new()
            .text("prefix", "text", "suffix")
            .text_opt("prefix", &Some("text"), "suffix")
            .text_opt("prefix", &None as &Option<String>, "suffix")
            .build();
        assert_eq!("prefixtextsuffixprefixtextsuffix", text);
    }

    #[test]
    fn escape_works() {
        let escaped = escape_markdown("a_b*c[d]e(f)g~h`i>j#k+l-m=n|o{p}q.r!s");
        assert_eq!(
            r#"a\_b\*c\[d\]e\(f\)g\~h\`i\>j\#k\+l\-m\=n\|o\{p\}q\.r\!s"#,
            escaped
        );
    }
}
