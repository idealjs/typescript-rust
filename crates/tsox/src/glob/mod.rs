use std::fmt;

#[derive(Clone, Debug)]
pub struct Glob {
    elems: Vec<Element>,
}

#[derive(Clone, Debug)]
enum Element {

    Slash,

    Literal(String),

    Star,

    AnyChar,

    StarStar,

    Group(Vec<Glob>),

    CharRange { negate: bool, low: char, high: char },
}

impl Glob {

    pub fn parse(pattern: &str) -> Result<Glob, String> {
        let (g, _rest) = parse_inner(pattern, false)?;
        Ok(g)
    }

    pub fn is_match(&self, input: &str) -> bool {
        match_elements(&self.elems, input)
    }
}

impl fmt::Display for Glob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for e in &self.elems {
            match e {
                Element::Slash => write!(f, "/")?,
                Element::Literal(s) => write!(f, "{}", s)?,
                Element::Star => write!(f, "*")?,
                Element::AnyChar => write!(f, "?")?,
                Element::StarStar => write!(f, "**")?,
                Element::Group(gs) => {
                    write!(f, "{{")?;
                    for (i, g) in gs.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",")?;
                        }
                        write!(f, "{}", g)?;
                    }
                    write!(f, "}}")?;
                }
                Element::CharRange { negate, low, high } => {
                    write!(f, "[")?;
                    if *negate {
                        write!(f, "!")?;
                    }
                    write!(f, "{}-{}", low, high)?;
                    write!(f, "]")?;
                }
            }
        }
        Ok(())
    }
}

fn parse_inner(pattern: &str, nested: bool) -> Result<(Glob, &str), String> {
    let mut elems = Vec::new();
    let mut chars = pattern.char_indices().peekable();
    let bytes = pattern.as_bytes();

    while let Some(&(idx, ch)) = chars.peek() {
        match ch {
            '/' => {
                chars.next();
                elems.push(Element::Slash);
            }
            '*' => {
                chars.next();
                if chars.peek().map(|&(_, c)| c) == Some('*') {

                    let last_is_slash = elems.last().map_or(false, |e| matches!(e, Element::Slash));
                    let next_after_starstar = pattern.as_bytes().get(idx + 2).copied();
                    if !last_is_slash
                        && next_after_starstar != Some(b'/')
                        && next_after_starstar.is_some()
                    {
                        return Err("** may only be adjacent to '/'".to_string());
                    }
                    if !last_is_slash && next_after_starstar.is_none() && !elems.is_empty() {
                        return Err("** may only be adjacent to '/'".to_string());
                    }
                    chars.next();
                    elems.push(Element::StarStar);
                } else {
                    elems.push(Element::Star);
                }
            }
            '?' => {
                chars.next();
                elems.push(Element::AnyChar);
            }
            '{' => {
                chars.next();
                let mut group = Vec::new();
                let mut rest = &pattern[idx + 1..];
                loop {
                    if rest.is_empty() {
                        return Err("unmatched '{'".to_string());
                    }
                    if rest.starts_with('}') {
                        rest = &rest[1..];
                        break;
                    }
                    let (g, remaining) = parse_inner(rest, true)?;
                    group.push(g);
                    rest = remaining;
                    if rest.starts_with(',') {
                        rest = &rest[1..];
                    } else if rest.starts_with('}') {
                        rest = &rest[1..];
                        break;
                    } else if rest.is_empty() {
                        return Err("unmatched '{'".to_string());
                    }
                }

                let _ = bytes;
                elems.push(Element::Group(group));

                let (inner, remaining) = parse_inner(rest, nested)?;
                elems.extend(inner.elems);
                return Ok((Glob { elems }, remaining));
            }
            '}' | ',' if nested => {
                let rest = &pattern[idx..];
                return Ok((Glob { elems }, rest));
            }
            '[' => {
                chars.next();
                let rest = &pattern[idx + 1..];
                if rest.is_empty() {
                    return Err("'[' patterns must be of the form [x-y]".to_string());
                }
                let negate = rest.starts_with('!');
                let after_negate = if negate { &rest[1..] } else { rest };
                if after_negate.is_empty() {
                    return Err("'[' patterns must be of the form [x-y]".to_string());
                }
                let low = after_negate.chars().next().unwrap();
                let after_low = &after_negate[low.len_utf8()..];
                if !after_low.starts_with('-') {
                    return Err("'[' patterns must be of the form [x-y]".to_string());
                }
                let after_dash = &after_low[1..];
                if after_dash.is_empty() {
                    return Err("'[' patterns must be of the form [x-y]".to_string());
                }
                let high = after_dash.chars().next().unwrap();
                let after_high = &after_dash[high.len_utf8()..];
                if !after_high.starts_with(']') {
                    return Err("'[' patterns must be of the form [x-y]".to_string());
                }
                let remaining = &after_high[1..];
                elems.push(Element::CharRange { negate, low, high });

                let consumed = pattern.len() - remaining.len();
                chars = pattern[consumed..].char_indices().peekable();
            }
            _ => {

                let special: &[char] = if nested {
                    &['*', '?', '{', '[', '/', '}', ',']
                } else {
                    &['*', '?', '{', '[', '/']
                };
                let start = idx;
                let mut end = idx;
                while let Some(&(i, c)) = chars.peek() {
                    if special.contains(&c) {
                        break;
                    }
                    end = i + c.len_utf8();
                    chars.next();
                }
                if end > start {
                    elems.push(Element::Literal(pattern[start..end].to_string()));
                } else {

                    chars.next();
                }
            }
        }
    }

    Ok((Glob { elems }, ""))
}

fn match_elements(elems: &[Element], input: &str) -> bool {
    let mut elems = elems.to_vec();
    let mut input = input.to_string();

    while !elems.is_empty() {
        let elem = elems.remove(0);
        match &elem {
            Element::Slash => {
                if input.is_empty() || !input.starts_with('/') {
                    return false;
                }
                while input.starts_with('/') {
                    input = input[1..].to_string();
                }
            }
            Element::StarStar => {

                if !elems.is_empty() {
                    elems.remove(0);
                }

                if elems.is_empty() {
                    return true;
                }

                while !input.is_empty() {
                    if match_elements(&elems, &input) {
                        return true;
                    }
                    let (_first, rest) = split(&input);
                    if rest.is_empty() {
                        return false;
                    }
                    input = rest.to_string();
                }
                return false;
            }
            Element::Literal(s) => {
                if !input.starts_with(s) {
                    return false;
                }
                input = input[s.len()..].to_string();
            }
            Element::Star => {
                let (seg_input, rest) = split(&input);

                let mut elem_end = elems.len();
                for (i, e) in elems.iter().enumerate() {
                    if matches!(e, Element::Slash) {
                        elem_end = i;
                        break;
                    }
                }
                let seg_elems: Vec<Element> = elems.drain(..elem_end).collect();

                if seg_elems.is_empty() {
                    input = rest.to_string();
                    continue;
                }

                let mut matched = false;
                for i in 0..=seg_input.len() {
                    if match_elements(&seg_elems, &seg_input[i..]) {
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    return false;
                }
                input = rest.to_string();
            }
            Element::AnyChar => {
                if input.is_empty() || input.starts_with('/') {
                    return false;
                }
                input = input[1..].to_string();
            }
            Element::Group(gs) => {

                for g in gs {
                    let mut branch = g.elems.clone();
                    branch.extend(elems.clone());
                    if match_elements(&branch, &input) {
                        return true;
                    }
                }
                return false;
            }
            Element::CharRange { negate, low, high } => {
                if input.is_empty() || input.starts_with('/') {
                    return false;
                }
                let c = input.chars().next().unwrap();
                let in_range = c >= *low && c <= *high;
                if in_range == *negate {
                    return false;
                }
                input = input[c.len_utf8()..].to_string();
            }
        }
    }

    input.is_empty()
}

fn split(input: &str) -> (&str, &str) {
    match input.find('/') {
        None => (input, ""),
        Some(i) => {
            let first = &input[..i];
            let after = &input[i..];

            let rest_start = after.bytes().position(|b| b != b'/').unwrap_or(after.len());
            (first, &after[rest_start..])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matches(pattern: &str, input: &str) -> bool {
        Glob::parse(pattern)
            .unwrap_or_else(|e| panic!("Failed to parse pattern '{}': {}", pattern, e))
            .is_match(input)
    }

    #[test]
    fn literal_match() {
        assert!(matches("foo", "foo"));
        assert!(!matches("foo", "bar"));
        assert!(!matches("foo", "foobar"));
    }

    #[test]
    fn star_match() {
        assert!(matches("*.ts", "foo.ts"));
        assert!(matches("*.ts", "bar.ts"));
        assert!(!matches("*.ts", "foo.js"));
        assert!(matches("a*", "abc"));
    }

    #[test]
    fn question_match() {
        assert!(matches("?.ts", "a.ts"));
        assert!(!matches("?.ts", "ab.ts"));
    }

    #[test]
    fn starstar_match() {
        assert!(matches("**/*.ts", "foo.ts"));
        assert!(matches("**/*.ts", "a/b/foo.ts"));
        assert!(matches("**/*.ts", "a/b/c/foo.ts"));
    }

    #[test]
    fn group_match() {
        assert!(matches("*.{ts,js}", "foo.ts"));
        assert!(matches("*.{ts,js}", "foo.js"));
        assert!(!matches("*.{ts,js}", "foo.json"));
    }

    #[test]
    fn char_range_match() {
        assert!(matches("example.[0-9]", "example.0"));
        assert!(matches("example.[0-9]", "example.9"));
        assert!(!matches("example.[0-9]", "example.a"));
    }

    #[test]
    fn negated_range_match() {
        assert!(matches("example.[!0-9]", "example.a"));
        assert!(!matches("example.[!0-9]", "example.0"));
    }

    #[test]
    fn slash_match() {
        assert!(matches("a/b", "a/b"));
        assert!(matches("a/b", "a//b"));
        assert!(!matches("a//b", "a/b"));
        assert!(!matches("a/b", "a/c"));
    }
}
