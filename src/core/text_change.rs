use super::text::TextRange;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextChange {
    pub range: TextRange,
    pub new_text: String,
}

impl TextChange {
    pub fn new(range: TextRange, new_text: impl Into<String>) -> Self {
        Self {
            range,
            new_text: new_text.into(),
        }
    }

    pub fn apply_to(&self, text: &str) -> String {
        let pos = self.range.pos();
        let end = self.range.end();
        let mut result = String::with_capacity(text.len() + self.new_text.len());
        result.push_str(&text[..pos]);
        result.push_str(&self.new_text);
        result.push_str(&text[end..]);
        result
    }
}

pub fn apply_bulk_edits(text: &str, edits: &[TextChange]) -> String {
    let mut result = String::with_capacity(text.len());
    let mut last_end = 0;
    for edit in edits {
        let start = edit.range.pos();
        if start != last_end {
            result.push_str(&text[last_end..start]);
        }
        result.push_str(&edit.new_text);
        last_end = edit.range.end();
    }
    result.push_str(&text[last_end..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_single_edit() {
        let text = "hello world";
        let edit = TextChange::new(TextRange::new(0, 5), "HELLO");
        assert_eq!(edit.apply_to(text), "HELLO world");
    }

    #[test]
    fn apply_bulk_edits_works() {
        let text = "abcdef";
        let edits = vec![
            TextChange::new(TextRange::new(0, 1), "A"),
            TextChange::new(TextRange::new(2, 3), "C"),
            TextChange::new(TextRange::new(5, 6), "F"),
        ];

        assert_eq!(apply_bulk_edits(text, &edits), "AbCdeF");
    }
}
