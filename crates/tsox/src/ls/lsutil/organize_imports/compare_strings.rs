use std::cmp::Ordering;

pub(super) fn compare_booleans(a: bool, b: bool) -> i32 {
    cmp_compare_i32(a as i32, b as i32)
}

pub(super) fn cmp_compare_i32(a: i32, b: i32) -> i32 {
    if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    }
}

pub(super) fn ord_to_i32(ord: Ordering) -> i32 {
    match ord {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

pub(super) fn next_rune(s: &str) -> (char, usize) {
    match s.chars().next() {
        Some(c) => (c, c.len_utf8()),
        None => ('\0', 0),
    }
}

pub(super) fn compare_organize_imports_natural_strings(
    a: &str,
    b: &str,
    case_sensitive: bool,
) -> i32 {
    let ord = compare_strings_numeric(&natural_collation_key(a), &natural_collation_key(b));
    if ord != 0 {
        return ord;
    }

    if case_sensitive {
        let ord = compare_organize_imports_case_upper_first(a, b);
        if ord != 0 {
            return ord;
        }
    }

    ord_to_i32(a.cmp(b))
}

pub(super) fn compare_organize_imports_unicode_strings(
    a: &str,
    b: &str,
    ignore_case: bool,
    case_first: super::super::user_preferences::OrganizeImportsCaseFirst,
    numeric: bool,
    accents: bool,
) -> i32 {
    let ord = compare_organize_imports_unicode_keys(
        &natural_collation_key(a),
        &natural_collation_key(b),
        numeric,
    );
    if ord != 0 {
        return ord;
    }

    if accents {
        let ord = compare_organize_imports_unicode_keys(
            &a.to_ascii_lowercase(),
            &b.to_ascii_lowercase(),
            numeric,
        );
        if ord != 0 {
            return ord;
        }
    }

    if !ignore_case {
        let ord = compare_organize_imports_case(a, b, case_first);
        if ord != 0 {
            return ord;
        }
    }

    ord_to_i32(a.cmp(b))
}

fn natural_collation_key(s: &str) -> String {
    s.to_ascii_lowercase()
        .chars()
        .filter(|&c| !is_combining_mark(c))
        .collect()
}

fn is_combining_mark(c: char) -> bool {
    let cp = c as u32;
    (0x0300..=0x036F).contains(&cp)
        || (0x0483..=0x0489).contains(&cp)
        || (0x0591..=0x05BD).contains(&cp)
        || cp == 0x05BF
        || (0x05C1..=0x05C2).contains(&cp)
        || (0x05C4..=0x05C5).contains(&cp)
        || cp == 0x05C7
        || (0x0610..=0x061A).contains(&cp)
        || (0x064B..=0x065F).contains(&cp)
        || cp == 0x0670
        || (0x06D6..=0x06DC).contains(&cp)
        || (0x06DF..=0x06E4).contains(&cp)
        || (0x06E7..=0x06E8).contains(&cp)
        || (0x06EA..=0x06ED).contains(&cp)
        || cp == 0x0711
        || (0x0730..=0x074A).contains(&cp)
}

pub(super) fn compare_organize_imports_unicode_keys(a: &str, b: &str, numeric: bool) -> i32 {
    if numeric {
        compare_strings_numeric(a, b)
    } else {
        ord_to_i32(a.cmp(b))
    }
}

fn compare_strings_numeric(a: &str, b: &str) -> i32 {
    let mut a = a;
    let mut b = b;
    while !a.is_empty() && !b.is_empty() {
        let a0 = a.as_bytes()[0];
        let b0 = b.as_bytes()[0];
        if is_ascii_digit(a0) && is_ascii_digit(b0) {
            let a_run_end = ascii_digit_run_end(a);
            let b_run_end = ascii_digit_run_end(b);
            let ord = compare_numeric_text(&a[..a_run_end], &b[..b_run_end]);
            if ord != 0 {
                return ord;
            }
            a = &a[a_run_end..];
            b = &b[b_run_end..];
            continue;
        }

        let (a_rune, a_size) = next_rune(a);
        let (b_rune, b_size) = next_rune(b);
        if a_rune != b_rune {
            return cmp_compare_i32(a_rune as i32, b_rune as i32);
        }
        a = &a[a_size..];
        b = &b[b_size..];
    }

    cmp_compare_i32(a.len() as i32, b.len() as i32)
}

fn is_ascii_digit(ch: u8) -> bool {
    ch.is_ascii_digit()
}

fn ascii_digit_run_end(s: &str) -> usize {
    s.bytes().take_while(|c| is_ascii_digit(*c)).count()
}

fn compare_numeric_text(a: &str, b: &str) -> i32 {
    let mut a_digits = a.trim_start_matches('0');
    let mut b_digits = b.trim_start_matches('0');
    if a_digits.is_empty() {
        a_digits = "0";
    }
    if b_digits.is_empty() {
        b_digits = "0";
    }

    if a_digits.len() != b_digits.len() {
        return cmp_compare_i32(a_digits.len() as i32, b_digits.len() as i32);
    }
    let ord = a_digits.cmp(b_digits);
    if ord != Ordering::Equal {
        return ord_to_i32(ord);
    }
    ord_to_i32(a.cmp(b))
}

fn compare_organize_imports_case_upper_first(a: &str, b: &str) -> i32 {
    compare_organize_imports_case(
        a,
        b,
        super::super::user_preferences::OrganizeImportsCaseFirst::Upper,
    )
}

pub(super) fn compare_organize_imports_case(
    a: &str,
    b: &str,
    case_first: super::super::user_preferences::OrganizeImportsCaseFirst,
) -> i32 {
    let a_runes: Vec<char> = a.chars().collect();
    let b_runes: Vec<char> = b.chars().collect();
    let min_len = a_runes.len().min(b_runes.len());

    for i in 0..min_len {
        let a_upper = a_runes[i].is_uppercase();
        let b_upper = b_runes[i].is_uppercase();
        if a_upper != b_upper {
            return match case_first {
                super::super::user_preferences::OrganizeImportsCaseFirst::Upper => {
                    if a_upper {
                        -1
                    } else {
                        1
                    }
                }
                super::super::user_preferences::OrganizeImportsCaseFirst::Lower => {
                    if !a_upper {
                        -1
                    } else {
                        1
                    }
                }
                super::super::user_preferences::OrganizeImportsCaseFirst::False => {
                    if a_upper {
                        1
                    } else {
                        -1
                    }
                }
            };
        }
    }

    cmp_compare_i32(a_runes.len() as i32, b_runes.len() as i32)
}
