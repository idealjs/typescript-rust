use std::collections::HashMap;

pub trait Named {
    fn name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct Index<T: Named + Clone> {
    pub entries: Vec<T>,
    index: HashMap<char, Vec<usize>>,
}

impl<T: Named + Clone> Default for Index<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            index: HashMap::new(),
        }
    }
}

impl<T: Named + Clone> Index<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn find(&self, name: &str, case_sensitive: bool) -> Vec<T> {
        if self.entries.is_empty() || name.is_empty() {
            return Vec::new();
        }
        let first_rune = match name.chars().next() {
            Some(c) => c,
            None => return Vec::new(),
        };
        let first_rune_upper = first_rune.to_ascii_uppercase();
        let candidates = match self.index.get(&first_rune_upper) {
            Some(c) => c,
            None => return Vec::new(),
        };

        let mut results = Vec::new();
        for &entry_index in candidates {
            let entry = &self.entries[entry_index];
            let entry_name = entry.name();
            if (case_sensitive && entry_name == name)
                || (!case_sensitive && eq_ignore_ascii_case(entry_name, name))
            {
                results.push(entry.clone());
            }
        }
        results
    }

    pub fn search_word_prefix(&self, prefix: &str) -> Vec<T> {
        if self.entries.is_empty() {
            return Vec::new();
        }
        if prefix.is_empty() {
            return self.entries.clone();
        }

        let prefix_lower = prefix.to_ascii_lowercase();
        let first_rune = match prefix_lower.chars().next() {
            Some(c) => c,
            None => return Vec::new(),
        };
        let first_rune_upper = first_rune.to_ascii_uppercase();
        let first_rune_lower = first_rune.to_ascii_lowercase();

        let name_starts = self
            .index
            .get(&first_rune_upper)
            .cloned()
            .unwrap_or_default();
        let word_starts = if first_rune_upper != first_rune_lower {
            self.index
                .get(&first_rune_lower)
                .cloned()
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let count = name_starts.len() + word_starts.len();
        if count == 0 {
            return Vec::new();
        }

        let mut results = Vec::with_capacity(count);
        for starts in [&name_starts, &word_starts] {
            for &i in starts {
                let entry = &self.entries[i];
                if contains_chars_in_order(entry.name(), &prefix_lower) {
                    results.push(entry.clone());
                }
            }
        }
        results
    }

    pub fn insert_as_words(&mut self, value: T) {
        let name = value.name().to_string();
        if name.is_empty() {
            panic!("Cannot index entry with empty name");
        }
        let entry_index = self.entries.len();
        self.entries.push(value);

        let indices = word_indices(&name);
        let mut seen_runes: HashMap<char, bool> = HashMap::new();

        for (i, &start) in indices.iter().enumerate() {
            let substr = &name[start..];
            let first_rune = match substr.chars().next() {
                Some(c) => c,
                None => continue,
            };
            if i == 0 {
                let upper = first_rune.to_ascii_uppercase();
                self.index.entry(upper).or_default().push(entry_index);
                seen_runes.insert(upper, true);
            } else {
                let lower = first_rune.to_ascii_lowercase();
                if !seen_runes.contains_key(&lower) {
                    self.index.entry(lower).or_default().push(entry_index);
                    seen_runes.insert(lower, true);
                }
            }
        }
    }

    pub fn clone_filtered(&self, filter: &dyn Fn(&T) -> bool) -> Index<T> {
        let mut new_idx = Index::<T>::new();
        new_idx.entries = Vec::with_capacity(self.entries.len());
        new_idx.index = HashMap::with_capacity(self.index.len());

        let mut old_to_new: HashMap<usize, usize> = HashMap::new();
        for (old_index, entry) in self.entries.iter().enumerate() {
            if filter(entry) {
                let new_index = new_idx.entries.len();
                new_idx.entries.push(entry.clone());
                old_to_new.insert(old_index, new_index);
            }
        }

        for (r, old_indices) in &self.index {
            let mut new_indices = Vec::with_capacity(old_indices.len());
            for &old_index in old_indices {
                if let Some(&new_index) = old_to_new.get(&old_index) {
                    new_indices.push(new_index);
                }
            }
            if !new_indices.is_empty() {
                new_idx.index.insert(*r, new_indices);
            }
        }

        new_idx
    }
}

fn contains_chars_in_order(s: &str, pattern: &str) -> bool {
    let str_lower = s.to_ascii_lowercase();
    let pattern_lower = pattern.to_ascii_lowercase();

    let mut pattern_chars = pattern_lower.chars().peekable();
    for ch in str_lower.chars() {
        if pattern_chars.peek() == Some(&ch) {
            pattern_chars.next();
        }
    }
    pattern_chars.peek().is_none()
}

fn eq_ignore_ascii_case(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

pub fn word_indices(s: &str) -> Vec<usize> {
    let mut indices = Vec::new();
    let bytes = s.as_bytes();
    for (byte_index, rune_value) in s.char_indices() {
        if byte_index == 0 {
            indices.push(byte_index);
            continue;
        }
        if rune_value == '_' {
            if byte_index + 1 < s.len() && bytes[byte_index + 1] != b'_' {
                indices.push(byte_index + 1);
            }
            continue;
        }
        if rune_value.is_ascii_uppercase() {
            let prev_is_lower = s[..byte_index]
                .chars()
                .rev()
                .next()
                .map(|c| c.is_ascii_lowercase())
                .unwrap_or(false);
            let next_is_lower = if byte_index + 1 < s.len() {
                s[byte_index + 1..]
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_lowercase())
                    .unwrap_or(false)
            } else {
                false
            };
            if prev_is_lower || next_is_lower {
                indices.push(byte_index);
            }
        }
    }
    indices
}

#[cfg(test)]
mod tests;
