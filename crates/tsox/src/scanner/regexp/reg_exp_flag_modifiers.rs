use super::*;

pub(super) const REG_EXP_FLAG_MODIFIERS: u16 = crate::scanner::REG_EXP_FLAG_I
    | crate::scanner::REG_EXP_FLAG_M
    | crate::scanner::REG_EXP_FLAG_S;

pub(super) fn char_to_reg_exp_flag(ch: char) -> u16 {
    match ch {
        'd' => crate::scanner::REG_EXP_FLAG_D,
        'g' => crate::scanner::REG_EXP_FLAG_G,
        'i' => crate::scanner::REG_EXP_FLAG_I,
        'm' => crate::scanner::REG_EXP_FLAG_M,
        's' => crate::scanner::REG_EXP_FLAG_S,
        'u' => crate::scanner::REG_EXP_FLAG_U,
        'v' => crate::scanner::REG_EXP_FLAG_V,
        'y' => crate::scanner::REG_EXP_FLAG_Y,
        _ => 0,
    }
}

pub(super) fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

pub(super) fn is_hex_digit(c: char) -> bool {
    c.is_ascii_hexdigit()
}

pub(super) fn is_octal_digit(c: char) -> bool {
    ('0'..='7').contains(&c)
}

pub(super) fn is_word_character(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

pub(super) fn is_ascii_letter(c: char) -> bool {
    c.is_ascii_alphabetic()
}

pub(super) fn decode_rune_at(text: &str, pos: usize) -> (char, usize) {
    match text[pos..].chars().next() {
        Some(c) => (c, c.len_utf8()),
        None => ('\0', 0),
    }
}

pub(super) fn decode_first_rune(s: &str) -> Option<(char, usize)> {
    let mut chars = s.chars();
    let c = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    Some((c, c.len_utf8()))
}

pub(super) fn compare_decimal_strings(a: &str, b: &str) -> i32 {
    let a = a.trim_start_matches('0');
    let b = b.trim_start_matches('0');
    let a = if a.is_empty() { "0" } else { a };
    let b = if b.is_empty() { "0" } else { b };
    if a.len() != b.len() {
        if a.len() < b.len() { -1 } else { 1 }
    } else {
        match a.cmp(b) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ClassSetExpressionType {
    Unknown,
    ClassUnion,
    ClassIntersection,
    ClassSubtraction,
}

#[derive(Clone, Debug)]
pub struct GroupNameReference {
    pub pos: usize,
    pub end: usize,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct DecimalEscapeValue {
    pub pos: usize,
    pub end: usize,
    pub value: i32,
}

pub struct RegExpParser<'a> {
    pub(super) text: &'a str,
    pub(super) pos: usize,
    pub(super) body_end: usize,
    #[allow(dead_code)]
    pub(super) flags: u16,
    pub(super) any_unicode_mode: bool,
    pub(super) unicode_sets_mode: bool,
    pub(super) annex_b: bool,
    pub(super) any_unicode_mode_or_non_annex_b: bool,
    pub(super) named_capture_groups: bool,
    pub(super) may_contain_strings: bool,
    pub(super) number_of_capturing_groups: i32,
    pub(super) group_specifiers: HashSet<String>,
    pub(super) group_name_references: Vec<GroupNameReference>,
    pub(super) decimal_escapes: Vec<DecimalEscapeValue>,
    pub(super) named_capturing_groups: Vec<HashSet<String>>,
    pub(super) errors: Vec<ScannerError>,
    pub(super) script_target: ScriptTarget,
}

impl<'a> RegExpParser<'a> {
    pub fn new(
        text: &'a str,
        body_start: usize,
        body_end: usize,
        flags: u16,
        named_capture_groups: bool,
        script_target: ScriptTarget,
    ) -> Self {
        let any_unicode_mode =
            (flags & (crate::scanner::REG_EXP_FLAG_U | crate::scanner::REG_EXP_FLAG_V)) != 0;
        let unicode_sets_mode = (flags & crate::scanner::REG_EXP_FLAG_V) != 0;
        let annex_b = !any_unicode_mode;
        Self {
            text,
            pos: body_start,
            body_end,
            flags,
            any_unicode_mode,
            unicode_sets_mode,
            annex_b,
            any_unicode_mode_or_non_annex_b: false,
            named_capture_groups,
            may_contain_strings: false,
            number_of_capturing_groups: 0,
            group_specifiers: HashSet::new(),
            group_name_references: Vec::new(),
            decimal_escapes: Vec::new(),
            named_capturing_groups: Vec::new(),
            errors: Vec::new(),
            script_target,
        }
    }

    #[allow(dead_code)]
    pub fn flags(&self) -> u16 {
        self.flags
    }

    pub fn errors(&self) -> &[ScannerError] {
        &self.errors
    }

    #[allow(dead_code)]
    pub fn take_errors(&mut self) -> Vec<ScannerError> {
        std::mem::take(&mut self.errors)
    }

    #[allow(dead_code)]
    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn set_pos(&mut self, v: usize) {
        self.pos = v;
    }

    pub fn run(&mut self) {
        self.any_unicode_mode_or_non_annex_b = self.any_unicode_mode || !self.annex_b;

        self.scan_disjunction(false);

        let group_name_references = self.group_name_references.clone();
        for reference in &group_name_references {
            if !self.group_specifiers.contains(&reference.name) {
                self.error(
                    diagnostics::THERE_IS_NO_CAPTURING_GROUP_NAMED_0_IN_THIS_REGULAR_EXPRESSION,
                    reference.pos,
                    reference.end - reference.pos,
                );
            }
        }

        let decimal_escapes = self.decimal_escapes.clone();
        for escape in &decimal_escapes {
            if escape.value > self.number_of_capturing_groups {
                if self.number_of_capturing_groups > 0 {
                    self.error(
                        diagnostics::THIS_BACKREFERENCE_REFERS_TO_A_GROUP_THAT_DOES_NOT_EXIST_THERE_ARE_ONLY_0_CAPTURING_GROUPS_IN_THIS_REGULAR_EXPRESSION,
                        escape.pos,
                        escape.end - escape.pos,
                    );
                } else {
                    self.error(
                        diagnostics::THIS_BACKREFERENCE_REFERS_TO_A_GROUP_THAT_DOES_NOT_EXIST_THERE_ARE_NO_CAPTURING_GROUPS_IN_THIS_REGULAR_EXPRESSION,
                        escape.pos,
                        escape.end - escape.pos,
                    );
                }
            }
        }
    }
}
