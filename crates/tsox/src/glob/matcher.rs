use super::element::Element;

pub(super) fn match_elements(elems: &[Element], input: &str) -> bool {
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
