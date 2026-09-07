use super::Glob;
use super::element::Element;

pub(super) fn parse_inner(pattern: &str, nested: bool) -> Result<(Glob, &str), String> {
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
