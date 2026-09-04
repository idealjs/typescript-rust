//! Regular-expression body validator, ported from
//! `internal/scanner/regexp.go`.
//!
//! `RegExpParser` validates the body of a regex literal (the text between
//! the opening `/` and the closing `/`) and reports the TS1501–TS1538
//! diagnostics. It is constructed by the scanner after it has located the
//! body boundaries and parsed the flag run; the parser then walks the body
//! and collects `ScannerError`s tagged with `DiagnosticKind::RegexMessage`.

use super::unicode_properties;
use super::{DiagnosticKind, ScannerError};
use crate::core::compiler_options::ScriptTarget;
use crate::diagnostics;
use crate::scanner::is_identifier_part;
use std::collections::HashSet;

// Bitmask of the subpattern-modifier flags (`i` | `m` | `s`), mirroring Go's
// `regularExpressionFlagsModifiers`. The individual flag constants are
// defined in `super::mod.rs` and are accessible here as a descendant module.
const REG_EXP_FLAG_MODIFIERS: u16 =
    super::REG_EXP_FLAG_I | super::REG_EXP_FLAG_M | super::REG_EXP_FLAG_S;

/// Maps a flag character to its bitmask bit, or `0` if it isn't a known
/// regular-expression flag. Mirrors Go's `charCodeToRegExpFlag`.
fn char_to_reg_exp_flag(ch: char) -> u16 {
    match ch {
        'd' => super::REG_EXP_FLAG_D,
        'g' => super::REG_EXP_FLAG_G,
        'i' => super::REG_EXP_FLAG_I,
        'm' => super::REG_EXP_FLAG_M,
        's' => super::REG_EXP_FLAG_S,
        'u' => super::REG_EXP_FLAG_U,
        'v' => super::REG_EXP_FLAG_V,
        'y' => super::REG_EXP_FLAG_Y,
        _ => 0,
    }
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_hex_digit(c: char) -> bool {
    c.is_ascii_hexdigit()
}

fn is_octal_digit(c: char) -> bool {
    ('0'..='7').contains(&c)
}

fn is_word_character(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn is_ascii_letter(c: char) -> bool {
    c.is_ascii_alphabetic()
}

/// Decode the first UTF-8 rune at `pos` in `text`. Returns `(char, size)`.
/// Since `&str` is always valid UTF-8, this always succeeds for `pos < text.len()`.
fn decode_rune_at(text: &str, pos: usize) -> (char, usize) {
    match text[pos..].chars().next() {
        Some(c) => (c, c.len_utf8()),
        None => ('\0', 0),
    }
}

/// Decode the first rune of `s`, returning `Some((char, size))` only when `s`
/// consists of exactly one rune (so it can be compared numerically as a class
/// range bound). Mirrors the `len(s) == size` guard in Go's
/// `stringutil.DecodeJSStringRune` usage. Surrogate pairing is skipped.
fn decode_first_rune(s: &str) -> Option<(char, usize)> {
    let mut chars = s.chars();
    let c = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    Some((c, c.len_utf8()))
}

/// Compare two digit strings numerically (after trimming leading zeros),
/// returning -1/0/1. Ported directly from Go's `compareDecimalStrings`.
fn compare_decimal_strings(a: &str, b: &str) -> i32 {
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

/// The kind of class set expression currently being scanned, mirroring Go's
/// `classSetExpressionType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ClassSetExpressionType {
    Unknown,
    ClassUnion,
    ClassIntersection,
    ClassSubtraction,
}

/// A reference to a named capturing group (`\k<name>`), recorded for
/// post-parse validation. Mirrors Go's `groupNameReference`.
#[derive(Clone, Debug)]
pub struct GroupNameReference {
    pub pos: usize,
    pub end: usize,
    pub name: String,
}

/// A numeric backreference (`\1`..`\9`), recorded for post-parse validation.
/// Mirrors Go's `decimalEscapeValue`.
#[derive(Clone, Debug)]
pub struct DecimalEscapeValue {
    pub pos: usize,
    pub end: usize,
    pub value: i32,
}

/// Validator for the body of a regular-expression literal.
///
/// Borrows the full source `text` and walks the region `[body_start, body_end)`
/// (the text between the opening and closing `/`). All positions reported in
/// errors are absolute, in the same coordinate system as the scanner.
pub struct RegExpParser<'a> {
    text: &'a str,
    pos: usize,
    body_end: usize,
    #[allow(dead_code)]
    flags: u16,
    any_unicode_mode: bool,
    unicode_sets_mode: bool,
    annex_b: bool,
    any_unicode_mode_or_non_annex_b: bool,
    named_capture_groups: bool,
    may_contain_strings: bool,
    number_of_capturing_groups: i32,
    group_specifiers: HashSet<String>,
    group_name_references: Vec<GroupNameReference>,
    decimal_escapes: Vec<DecimalEscapeValue>,
    named_capturing_groups: Vec<HashSet<String>>,
    errors: Vec<ScannerError>,
    script_target: ScriptTarget,
}

impl<'a> RegExpParser<'a> {
    /// Construct a new parser over the body region `[body_start, body_end)`
    /// of `text`. `flags` is the regex flag bitmask (using the `REG_EXP_FLAG_*`
    /// constants from `super::mod.rs`). `named_capture_groups` indicates
    /// whether the body contains a `(?<` named-capture group (pre-scanned by
    /// the caller, mirroring Go's `reScanSlashToken`).
    pub fn new(
        text: &'a str,
        body_start: usize,
        body_end: usize,
        flags: u16,
        named_capture_groups: bool,
        script_target: ScriptTarget,
    ) -> Self {
        let any_unicode_mode = (flags & (super::REG_EXP_FLAG_U | super::REG_EXP_FLAG_V)) != 0;
        let unicode_sets_mode = (flags & super::REG_EXP_FLAG_V) != 0;
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

    // ────────────────────────────────────────────────────────────────────
    // Position helpers
    // ────────────────────────────────────────────────────────────────────

    #[allow(dead_code)]
    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn set_pos(&mut self, v: usize) {
        self.pos = v;
    }

    /// Advance `pos` by `n` (may be negative). Saturates at 0 for negative
    /// underflow, mirroring the safety of Go's signed `incPos`.
    fn inc_pos(&mut self, n: i32) {
        if n >= 0 {
            self.pos = self.pos.wrapping_add(n as usize);
        } else {
            self.pos = self.pos.saturating_sub((-n) as usize);
        }
    }

    /// Returns the byte at `pos` as a `char`, or `'\0'` when `pos >= body_end`.
    /// This mirrors Go's `Scanner.char()` returning `-1` at EOF; `'\0'` is
    /// used because Rust's `char` is unsigned.
    fn char(&self) -> char {
        if self.pos < self.body_end {
            self.text.as_bytes()[self.pos] as char
        } else {
            '\0'
        }
    }

    /// Returns the byte at `pos + offset` as a `char`, or `'\0'` if out of
    /// range. Mirrors Go's `Scanner.charAt(offset)`.
    fn char_at(&self, offset: usize) -> char {
        match self.pos.checked_add(offset) {
            Some(p) if p < self.body_end => self.text.as_bytes()[p] as char,
            _ => '\0',
        }
    }

    #[allow(dead_code)]
    pub fn text(&self) -> &str {
        self.text
    }

    /// Returns the 2 bytes at `pos` as a `[u8; 2]` when at least 2 bytes remain
    /// before `body_end`, else `None`. Used to detect the `--`/`&&` operator
    /// pairs without slicing the `&str` (which could split a multi-byte rune).
    fn two_chars_at(&self, pos: usize) -> Option<[u8; 2]> {
        if pos + 1 < self.body_end {
            let bytes = self.text.as_bytes();
            Some([bytes[pos], bytes[pos + 1]])
        } else {
            None
        }
    }

    fn is_class_content_exit(&self, ch: char) -> bool {
        ch == ']' || self.pos >= self.body_end
    }

    // ────────────────────────────────────────────────────────────────────
    // Error reporting
    // ────────────────────────────────────────────────────────────────────

    /// Push a `RegexMessage` diagnostic. The Go signature takes formatting
    /// args (e.g. the offending character), but here we store only the
    /// `Message` key — formatting/args and spelling suggestions are deferred
    /// to the diagnostic writer, per the porting spec.
    fn error(&mut self, msg: diagnostics::Message, pos: usize, length: usize) {
        self.errors.push(ScannerError {
            kind: DiagnosticKind::RegexMessage(msg),
            pos,
            length,
        });
    }

    fn scan_expected_char(&mut self, ch: char) {
        if self.char() == ch {
            self.inc_pos(1);
        } else {
            self.error(diagnostics::X_0_EXPECTED, self.pos, 0);
        }
    }

    /// Check target-gated availability of a regex flag (`d`→ES2022,
    /// `s`→ES2018, `v`→ES2024). Mirrors Go's `checkRegularExpressionFlagAvailability`.
    fn check_regular_expression_flag_availability(&mut self, flag: u16, pos: usize, size: usize) {
        let available_from = match flag {
            super::REG_EXP_FLAG_D => Some(ScriptTarget::ES2022),
            super::REG_EXP_FLAG_S => Some(ScriptTarget::ES2018),
            super::REG_EXP_FLAG_V => Some(ScriptTarget::ES2024),
            _ => None,
        };
        if let Some(target) = available_from {
            if self.script_target < target {
                self.error(
                    diagnostics::THIS_REGULAR_EXPRESSION_FLAG_IS_ONLY_AVAILABLE_WHEN_TARGETING_0_OR_LATER,
                    pos,
                    size,
                );
            }
        }
    }

    // ────────────────────────────────────────────────────────────────────
    // Entry point
    // ────────────────────────────────────────────────────────────────────

    /// Run the validator over the body. Mirrors Go's `regExpParser.run`.
    pub fn run(&mut self) {
        // Regular expressions are checked more strictly when either in 'u' or
        // 'v' mode, or when not using the looser interpretation of the syntax
        // from ECMA-262 Annex B.
        self.any_unicode_mode_or_non_annex_b = self.any_unicode_mode || !self.annex_b;

        self.scan_disjunction(false);

        // Validate named group references.
        let group_name_references = self.group_name_references.clone();
        for reference in &group_name_references {
            if !self.group_specifiers.contains(&reference.name) {
                self.error(
                    diagnostics::THERE_IS_NO_CAPTURING_GROUP_NAMED_0_IN_THIS_REGULAR_EXPRESSION,
                    reference.pos,
                    reference.end - reference.pos,
                );
                // Spelling suggestions are skipped per the porting spec.
            }
        }

        // Validate numeric backreferences.
        let decimal_escapes = self.decimal_escapes.clone();
        for escape in &decimal_escapes {
            // Although a DecimalEscape with a value greater than the number of
            // capturing groups is treated as either a LegacyOctalEscapeSequence
            // or an IdentityEscape in Annex B, an error is nevertheless
            // reported since it's most likely a mistake.
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

    // ────────────────────────────────────────────────────────────────────
    // Disjunction / Alternative
    // ────────────────────────────────────────────────────────────────────

    // Disjunction ::= Alternative ('|' Alternative)*
    fn scan_disjunction(&mut self, is_in_group: bool) {
        loop {
            self.named_capturing_groups.push(HashSet::new());
            self.scan_alternative(is_in_group);
            self.named_capturing_groups.pop();
            if self.char() != '|' {
                return;
            }
            self.inc_pos(1);
        }
    }

    // Alternative ::= Term*
    fn scan_alternative(&mut self, is_in_group: bool) {
        let mut is_previous_term_quantifiable = false;
        while self.pos < self.body_end {
            let start = self.pos;
            let ch = self.char();
            match ch {
                '^' | '$' => {
                    self.inc_pos(1);
                    is_previous_term_quantifiable = false;
                }
                '\\' => {
                    self.inc_pos(1);
                    match self.char() {
                        'b' | 'B' => {
                            self.inc_pos(1);
                            is_previous_term_quantifiable = false;
                        }
                        _ => {
                            self.scan_atom_escape();
                            is_previous_term_quantifiable = true;
                        }
                    }
                }
                '(' => {
                    self.inc_pos(1);
                    if self.char() == '?' {
                        self.inc_pos(1);
                        match self.char() {
                            '=' | '!' => {
                                self.inc_pos(1);
                                // In Annex B, `(?=Disjunction)` and `(?!Disjunction)`
                                // are quantifiable.
                                is_previous_term_quantifiable =
                                    !self.any_unicode_mode_or_non_annex_b;
                            }
                            '<' => {
                                let group_name_start = self.pos;
                                self.inc_pos(1);
                                match self.char() {
                                    '=' | '!' => {
                                        self.inc_pos(1);
                                        is_previous_term_quantifiable = false;
                                    }
                                    _ => {
                                        self.scan_group_name(false);
                                        self.scan_expected_char('>');
                                        if self.script_target < ScriptTarget::ES2018 {
                                            self.error(
                                                diagnostics::NAMED_CAPTURING_GROUPS_ARE_ONLY_AVAILABLE_WHEN_TARGETING_ES2018_OR_LATER,
                                                group_name_start,
                                                self.pos - group_name_start,
                                            );
                                        }
                                        self.number_of_capturing_groups += 1;
                                        is_previous_term_quantifiable = true;
                                    }
                                }
                            }
                            _ => {
                                let flags_start = self.pos;
                                let set_flags = self.scan_pattern_modifiers(0);
                                if self.char() == '-' {
                                    self.inc_pos(1);
                                    self.scan_pattern_modifiers(set_flags);
                                    if self.pos == flags_start + 1 {
                                        self.error(
                                            diagnostics::SUBPATTERN_FLAGS_MUST_BE_PRESENT_WHEN_THERE_IS_A_MINUS_SIGN,
                                            flags_start,
                                            self.pos - flags_start,
                                        );
                                    }
                                }
                                self.scan_expected_char(':');
                                is_previous_term_quantifiable = true;
                            }
                        }
                    } else {
                        self.number_of_capturing_groups += 1;
                        is_previous_term_quantifiable = true;
                    }
                    self.scan_disjunction(true);
                    self.scan_expected_char(')');
                }
                '{' => {
                    self.inc_pos(1);
                    let digits_start = self.pos;
                    let min_str = self.scan_digits();
                    if !self.any_unicode_mode_or_non_annex_b && min_str.is_empty() {
                        is_previous_term_quantifiable = true;
                        continue;
                    }
                    if self.char() == ',' {
                        self.inc_pos(1);
                        let max_str = self.scan_digits();
                        if min_str.is_empty() {
                            if !max_str.is_empty() || self.char() == '}' {
                                self.error(
                                    diagnostics::INCOMPLETE_QUANTIFIER_DIGIT_EXPECTED,
                                    digits_start,
                                    0,
                                );
                            } else {
                                self.error(
                                    diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                                    start,
                                    1,
                                );
                                is_previous_term_quantifiable = true;
                                continue;
                            }
                        } else if !max_str.is_empty() {
                            if compare_decimal_strings(&min_str, &max_str) > 0
                                && (self.any_unicode_mode_or_non_annex_b || self.char() == '}')
                            {
                                self.error(
                                    diagnostics::NUMBERS_OUT_OF_ORDER_IN_QUANTIFIER,
                                    digits_start,
                                    self.pos - digits_start,
                                );
                            }
                        }
                    } else if min_str.is_empty() {
                        if self.any_unicode_mode_or_non_annex_b {
                            self.error(
                                diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                                start,
                                1,
                            );
                        }
                        is_previous_term_quantifiable = true;
                        continue;
                    }
                    if self.char() != '}' {
                        if self.any_unicode_mode_or_non_annex_b {
                            self.error(diagnostics::X_0_EXPECTED, self.pos, 0);
                            self.inc_pos(-1);
                        } else {
                            is_previous_term_quantifiable = true;
                            continue;
                        }
                    }
                    // Fallthrough to the quantifier handling (shared with
                    // '*', '+', '?').
                    self.inc_pos(1);
                    if self.char() == '?' {
                        // Non-greedy
                        self.inc_pos(1);
                    }
                    if !is_previous_term_quantifiable {
                        self.error(
                            diagnostics::THERE_IS_NOTHING_AVAILABLE_FOR_REPETITION,
                            start,
                            self.pos - start,
                        );
                    }
                    is_previous_term_quantifiable = false;
                }
                '*' | '+' | '?' => {
                    self.inc_pos(1);
                    if self.char() == '?' {
                        // Non-greedy
                        self.inc_pos(1);
                    }
                    if !is_previous_term_quantifiable {
                        self.error(
                            diagnostics::THERE_IS_NOTHING_AVAILABLE_FOR_REPETITION,
                            start,
                            self.pos - start,
                        );
                    }
                    is_previous_term_quantifiable = false;
                }
                '.' => {
                    self.inc_pos(1);
                    is_previous_term_quantifiable = true;
                }
                '[' => {
                    self.inc_pos(1);
                    if self.unicode_sets_mode {
                        self.scan_class_set_expression();
                    } else {
                        self.scan_class_ranges();
                    }
                    self.scan_expected_char(']');
                    is_previous_term_quantifiable = true;
                }
                ')' => {
                    if is_in_group {
                        return;
                    }
                    // Fallthrough: same handling as ']' / '}'.
                    if self.any_unicode_mode_or_non_annex_b || ch == ')' {
                        self.error(
                            diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                            self.pos,
                            1,
                        );
                    }
                    self.inc_pos(1);
                    is_previous_term_quantifiable = true;
                }
                ']' | '}' => {
                    if self.any_unicode_mode_or_non_annex_b || ch == ')' {
                        self.error(
                            diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                            self.pos,
                            1,
                        );
                    }
                    self.inc_pos(1);
                    is_previous_term_quantifiable = true;
                }
                '/' | '|' => return,
                _ => {
                    self.scan_source_character();
                    is_previous_term_quantifiable = true;
                }
            }
        }
    }

    fn scan_pattern_modifiers(&mut self, curr_flags: u16) -> u16 {
        let mut curr = curr_flags;
        while self.pos < self.body_end {
            let (ch, size) = decode_rune_at(self.text, self.pos);
            if !is_identifier_part(ch) {
                break;
            }
            let flag = char_to_reg_exp_flag(ch);
            if flag == 0 {
                self.error(diagnostics::UNKNOWN_REGULAR_EXPRESSION_FLAG, self.pos, size);
            } else if curr & flag != 0 {
                self.error(
                    diagnostics::DUPLICATE_REGULAR_EXPRESSION_FLAG,
                    self.pos,
                    size,
                );
            } else if flag & REG_EXP_FLAG_MODIFIERS == 0 {
                self.error(
                    diagnostics::THIS_REGULAR_EXPRESSION_FLAG_CANNOT_BE_TOGGLED_WITHIN_A_SUBPATTERN,
                    self.pos,
                    size,
                );
            } else {
                curr |= flag;
                self.check_regular_expression_flag_availability(flag, self.pos, size);
            }
            self.inc_pos(size as i32);
        }
        curr
    }

    // ────────────────────────────────────────────────────────────────────
    // Digit / word scanning
    // ────────────────────────────────────────────────────────────────────

    /// Scan a run of decimal digits and return the matched text. Mirrors
    /// Go's `scanDigits`, but returns the string instead of stashing it in
    /// `scanner.tokenValue`.
    fn scan_digits(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.body_end && is_digit(self.char()) {
            self.inc_pos(1);
        }
        self.text[start..self.pos].to_string()
    }

    /// Scan a run of word characters (`[A-Za-z0-9_]`) and return the matched
    /// text. Mirrors Go's `scanWordCharacters`.
    fn scan_word_characters(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.body_end && is_word_character(self.char()) {
            self.inc_pos(1);
        }
        self.text[start..self.pos].to_string()
    }

    // ────────────────────────────────────────────────────────────────────
    // AtomEscape / DecimalEscape / CharacterEscape
    // ────────────────────────────────────────────────────────────────────

    // AtomEscape ::=
    //   | DecimalEscape
    //   | CharacterClassEscape
    //   | CharacterEscape
    //   | 'k<' RegExpIdentifierName '>'
    fn scan_atom_escape(&mut self) {
        // Precondition: pos is at the char following a backslash.
        let ch = self.char();
        if ch == 'k' {
            self.inc_pos(1);
            if self.char() == '<' {
                self.inc_pos(1);
                self.scan_group_name(true);
                self.scan_expected_char('>');
            } else if self.any_unicode_mode_or_non_annex_b || self.named_capture_groups {
                self.error(
                    diagnostics::X_K_MUST_BE_FOLLOWED_BY_A_CAPTURING_GROUP_NAME_ENCLOSED_IN_ANGLE_BRACKETS,
                    self.pos - 2,
                    2,
                );
            }
            return;
        }
        if ch == 'q' && self.unicode_sets_mode {
            self.inc_pos(1);
            self.error(
                diagnostics::X_Q_IS_ONLY_AVAILABLE_INSIDE_CHARACTER_CLASS,
                self.pos - 2,
                2,
            );
            return;
        }
        // default
        if !self.scan_character_class_escape() && !self.scan_decimal_escape() {
            // Regex literals cannot contain line breaks here, so a character
            // escape must consume something.
            let _ = self.scan_character_escape(true);
        }
    }

    // DecimalEscape ::= [1-9] [0-9]*
    fn scan_decimal_escape(&mut self) -> bool {
        // Precondition: pos is at the char following a backslash.
        let ch = self.char();
        if ('1'..='9').contains(&ch) {
            let start = self.pos;
            let digits = self.scan_digits();
            let val = digits.parse::<i32>().unwrap_or(i32::MAX);
            self.decimal_escapes.push(DecimalEscapeValue {
                pos: start,
                end: self.pos,
                value: val,
            });
            return true;
        }
        false
    }

    // CharacterEscape ::=
    //   | `c` ControlLetter
    //   | IdentityEscape
    //   | (Other sequences handled by `scan_escape_sequence`)
    fn scan_character_escape(&mut self, atom_escape: bool) -> String {
        // Precondition: pos is at the char following a backslash.
        if self.pos >= self.body_end {
            self.error(diagnostics::UNDETERMINED_CHARACTER_ESCAPE, self.pos - 1, 1);
            return "\\".to_string();
        }
        let ch = self.char();
        match ch {
            'c' => {
                self.inc_pos(1);
                let c2 = self.char();
                if is_ascii_letter(c2) {
                    self.inc_pos(1);
                    let code = (c2 as u32) & 0x1f;
                    return char::from_u32(code)
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| code.to_string());
                }
                if self.any_unicode_mode_or_non_annex_b {
                    self.error(
                        diagnostics::X_C_MUST_BE_FOLLOWED_BY_AN_ASCII_LETTER,
                        self.pos - 2,
                        2,
                    );
                } else if atom_escape {
                    self.inc_pos(-1);
                    return "\\".to_string();
                }
                c2.to_string()
            }
            '^' | '$' | '/' | '\\' | '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}'
            | '|' => {
                self.inc_pos(1);
                ch.to_string()
            }
            _ => {
                // Back up to include the backslash for scan_escape_sequence.
                self.inc_pos(-1);
                self.scan_escape_sequence(self.annex_b, self.any_unicode_mode, atom_escape)
            }
        }
    }

    /// A simplified `scanEscapeSequence` for the regex parser. Starts at `\`,
    /// advances past it, and returns the escaped character as a `String`.
    ///
    /// Handles `\0` (null when not followed by a digit), legacy octal escapes
    /// (`\0`+digit, `\1`–`\7`), `\8`/`\9`, the named escapes
    /// (`\b \t \n \v \f \r \' \""`), `\xHH`, `\uHHHH`, `\u{...}`, and the
    /// default identity escape. `\cX` is handled by `scan_character_escape`.
    /// Reports TS1535/TS1536/TS1537/TS1538 as appropriate. Surrogate pairing
    /// is skipped (not needed for validation).
    fn scan_escape_sequence(
        &mut self,
        annex_b: bool,
        any_unicode_mode: bool,
        atom_escape: bool,
    ) -> String {
        // Precondition: pos is at the backslash.
        let start = self.pos;
        self.inc_pos(1); // skip backslash
        if self.pos >= self.body_end {
            self.error(diagnostics::UNDETERMINED_CHARACTER_ESCAPE, start, 1);
            return "\\".to_string();
        }
        let ch = self.char();
        self.inc_pos(1); // skip the escaped char's first byte
        match ch {
            '0' => {
                // '\0' not followed by a digit is the NUL character.
                if !is_digit(self.char()) {
                    return "\0".to_string();
                }
                // '\0' + digit → legacy octal escape; consume up to 2 more
                // octal digits (mirrors the '1'-'3' → '4'-'7' fallthrough chain).
                if is_octal_digit(self.char()) {
                    self.inc_pos(1);
                }
                if is_octal_digit(self.char()) {
                    self.inc_pos(1);
                }
                self.report_octal_escape(start, '0', atom_escape);
                self.text[start..self.pos].to_string()
            }
            '1' | '2' | '3' => {
                if is_octal_digit(self.char()) {
                    self.inc_pos(1);
                }
                if is_octal_digit(self.char()) {
                    self.inc_pos(1);
                }
                self.report_octal_escape(start, ch, atom_escape);
                self.text[start..self.pos].to_string()
            }
            '4' | '5' | '6' | '7' => {
                if is_octal_digit(self.char()) {
                    self.inc_pos(1);
                }
                self.report_octal_escape(start, ch, atom_escape);
                self.text[start..self.pos].to_string()
            }
            '8' | '9' => {
                if !atom_escape {
                    self.error(
                        diagnostics::DECIMAL_ESCAPE_SEQUENCES_AND_BACKREFERENCES_ARE_NOT_ALLOWED_IN_A_CHARACTER_CLASS,
                        start,
                        self.pos - start,
                    );
                }
                ch.to_string()
            }
            'b' => "\u{0008}".to_string(),
            't' => "\t".to_string(),
            'n' => "\n".to_string(),
            'v' => "\u{000B}".to_string(),
            'f' => "\u{000C}".to_string(),
            'r' => "\r".to_string(),
            '\'' => "'".to_string(),
            '"' => "\"".to_string(),
            'x' => {
                let hex_start = self.pos;
                for _ in 0..2 {
                    if is_hex_digit(self.char()) {
                        self.inc_pos(1);
                    } else {
                        break;
                    }
                }
                let hex = &self.text[hex_start..self.pos];
                if let Ok(n) = u32::from_str_radix(hex, 16) {
                    if let Some(c) = char::from_u32(n) {
                        return c.to_string();
                    }
                }
                self.text[start..self.pos].to_string()
            }
            'u' => {
                if self.char() == '{' {
                    // Extended '\u{...}' escape.
                    self.inc_pos(1); // skip '{'
                    let hex_start = self.pos;
                    while is_hex_digit(self.char()) {
                        self.inc_pos(1);
                    }
                    let hex = &self.text[hex_start..self.pos];
                    if self.char() == '}' {
                        self.inc_pos(1);
                    }
                    if !any_unicode_mode {
                        self.error(
                            diagnostics::UNICODE_ESCAPE_SEQUENCES_ARE_ONLY_AVAILABLE_WHEN_THE_UNICODE_U_FLAG_OR_THE_UNICODE_SETS_V_FLAG_IS_SET,
                            start,
                            self.pos - start,
                        );
                    }
                    if let Ok(n) = u32::from_str_radix(hex, 16) {
                        if let Some(c) = char::from_u32(n) {
                            return c.to_string();
                        }
                    }
                    self.text[start..self.pos].to_string()
                } else {
                    // '\uHHHH'
                    let hex_start = self.pos;
                    for _ in 0..4 {
                        if is_hex_digit(self.char()) {
                            self.inc_pos(1);
                        } else {
                            break;
                        }
                    }
                    let hex = &self.text[hex_start..self.pos];
                    if hex.len() == 4 {
                        if let Ok(n) = u32::from_str_radix(hex, 16) {
                            if let Some(c) = char::from_u32(n) {
                                return c.to_string();
                            }
                        }
                    }
                    self.text[start..self.pos].to_string()
                }
            }
            '\r' => {
                // LineContinuation: backslash + line terminator is the empty
                // code unit sequence.
                if self.char() == '\n' {
                    self.inc_pos(1);
                }
                String::new()
            }
            '\n' => String::new(),
            _ => {
                // `ch` was read as a single byte; for multi-byte UTF-8 we must
                // decode the full rune starting at the byte after the backslash.
                let byte_pos = start + 1;
                self.set_pos(byte_pos);
                let (c, size) = decode_rune_at(self.text, byte_pos);
                self.set_pos(byte_pos + size);
                if c == '\u{2028}' || c == '\u{2029}' {
                    return String::new();
                }
                if any_unicode_mode || (!annex_b && is_identifier_part(c)) {
                    self.error(
                        diagnostics::THIS_CHARACTER_CANNOT_BE_ESCAPED_IN_A_REGULAR_EXPRESSION,
                        start,
                        self.pos - start,
                    );
                }
                c.to_string()
            }
        }
    }

    /// Report the TS1536 octal-in-character-class diagnostic for a legacy
    /// octal escape. Mirrors the regex-context branch of Go's
    /// `scanEscapeSequence` octal handling. The string-literal octal error
    /// (TS1197) is out of scope for this simplified regex parser.
    fn report_octal_escape(&mut self, start: usize, ch: char, atom_escape: bool) {
        if !atom_escape && ch != '0' {
            self.error(
                diagnostics::OCTAL_ESCAPE_SEQUENCES_AND_BACKREFERENCES_ARE_NOT_ALLOWED_IN_A_CHARACTER_CLASS_IF_THIS_WAS_INTENDED_AS_AN_ESCAPE_SEQUENCE_USE_THE_SYNTAX_0_INSTEAD,
                start,
                self.pos - start,
            );
        }
    }

    // ────────────────────────────────────────────────────────────────────
    // Group names
    // ────────────────────────────────────────────────────────────────────

    fn scan_group_name(&mut self, is_reference: bool) {
        // Precondition: pos is at the char following '<'.
        let token_start = self.pos;
        let name = self.scan_identifier_name();
        if self.pos == token_start {
            self.error(diagnostics::EXPECTED_A_CAPTURING_GROUP_NAME, self.pos, 0);
        } else if is_reference {
            self.group_name_references.push(GroupNameReference {
                pos: token_start,
                end: self.pos,
                name,
            });
        } else if self.named_capturing_groups_contains(&name) {
            self.error(
                diagnostics::NAMED_CAPTURING_GROUPS_WITH_THE_SAME_NAME_MUST_BE_MUTUALLY_EXCLUSIVE_TO_EACH_OTHER,
                token_start,
                self.pos - token_start,
            );
        } else {
            if let Some(last) = self.named_capturing_groups.last_mut() {
                last.insert(name.clone());
            }
            self.group_specifiers.insert(name);
        }
    }

    fn named_capturing_groups_contains(&self, name: &str) -> bool {
        self.named_capturing_groups.iter().any(|g| g.contains(name))
    }

    /// A simplified identifier scan that reads `is_identifier_part` characters,
    /// standing in for Go's `Scanner.scanIdentifier`. Unicode escape sequences
    /// in identifiers are not handled (sufficient for group-name validation).
    fn scan_identifier_name(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.body_end {
            let (c, size) = decode_rune_at(self.text, self.pos);
            if !is_identifier_part(c) {
                break;
            }
            self.pos += size;
        }
        self.text[start..self.pos].to_string()
    }

    // ────────────────────────────────────────────────────────────────────
    // Character classes (non-Unicode-Sets: ClassRanges)
    // ────────────────────────────────────────────────────────────────────

    // ClassRanges ::= '^'? (ClassAtom ('-' ClassAtom)?)*
    fn scan_class_ranges(&mut self) {
        // Precondition: pos is at the char following '['.
        if self.char() == '^' {
            self.inc_pos(1);
        }
        while self.pos < self.body_end {
            let ch = self.char();
            if self.is_class_content_exit(ch) {
                return;
            }
            let min_start = self.pos;
            let min_character = self.scan_class_atom();
            if self.char() == '-' {
                self.inc_pos(1);
                let ch2 = self.char();
                if self.is_class_content_exit(ch2) {
                    return;
                }
                if min_character.is_empty() && self.any_unicode_mode_or_non_annex_b {
                    self.error(
                        diagnostics::A_CHARACTER_CLASS_RANGE_MUST_NOT_BE_BOUNDED_BY_ANOTHER_CHARACTER_CLASS,
                        min_start,
                        self.pos - 1 - min_start,
                    );
                }
                let max_start = self.pos;
                let max_character = self.scan_class_atom();
                if max_character.is_empty() && self.any_unicode_mode_or_non_annex_b {
                    self.error(
                        diagnostics::A_CHARACTER_CLASS_RANGE_MUST_NOT_BE_BOUNDED_BY_ANOTHER_CHARACTER_CLASS,
                        max_start,
                        self.pos - max_start,
                    );
                    continue;
                }
                if min_character.is_empty() {
                    continue;
                }
                if let (Some((min_c, min_size)), Some((max_c, max_size))) = (
                    decode_first_rune(&min_character),
                    decode_first_rune(&max_character),
                ) {
                    if min_character.len() == min_size
                        && max_character.len() == max_size
                        && (min_c as u32) > (max_c as u32)
                    {
                        self.error(
                            diagnostics::RANGE_OUT_OF_ORDER_IN_CHARACTER_CLASS,
                            min_start,
                            self.pos - min_start,
                        );
                    }
                }
            }
        }
    }

    // ────────────────────────────────────────────────────────────────────
    // Character classes (Unicode-Sets: ClassSetExpression)
    // ────────────────────────────────────────────────────────────────────

    // ClassSetExpression ::= '^'? (ClassUnion | ClassIntersection | ClassSubtraction)
    fn scan_class_set_expression(&mut self) {
        // Precondition: pos is at the char following '['.
        let mut is_character_complement = false;
        if self.char() == '^' {
            self.inc_pos(1);
            is_character_complement = true;
        }
        let mut expression_may_contain_strings = false;
        let ch = self.char();
        if self.is_class_content_exit(ch) {
            return;
        }
        let mut start = self.pos;
        let operand: String;
        let two = self.two_chars_at(self.pos);
        if matches!(two, Some([b'-', b'-']) | Some([b'&', b'&'])) {
            self.error(diagnostics::EXPECTED_A_CLASS_SET_OPERAND, self.pos, 0);
            self.may_contain_strings = false;
            operand = String::new();
        } else {
            operand = self.scan_class_set_operand();
        }

        match self.char() {
            '-' => {
                if self.pos + 1 < self.body_end && self.char_at(1) == '-' {
                    if is_character_complement && self.may_contain_strings {
                        self.error(
                            diagnostics::ANYTHING_THAT_WOULD_POSSIBLY_MATCH_MORE_THAN_A_SINGLE_CHARACTER_IS_INVALID_INSIDE_A_NEGATED_CHARACTER_CLASS,
                            start,
                            self.pos - start,
                        );
                    }
                    expression_may_contain_strings = self.may_contain_strings;
                    self.scan_class_set_sub_expression(ClassSetExpressionType::ClassSubtraction);
                    self.may_contain_strings =
                        !is_character_complement && expression_may_contain_strings;
                    return;
                }
            }
            '&' => {
                if self.pos + 1 < self.body_end && self.char_at(1) == '&' {
                    self.scan_class_set_sub_expression(ClassSetExpressionType::ClassIntersection);
                    if is_character_complement && self.may_contain_strings {
                        self.error(
                            diagnostics::ANYTHING_THAT_WOULD_POSSIBLY_MATCH_MORE_THAN_A_SINGLE_CHARACTER_IS_INVALID_INSIDE_A_NEGATED_CHARACTER_CLASS,
                            start,
                            self.pos - start,
                        );
                    }
                    expression_may_contain_strings = self.may_contain_strings;
                    self.may_contain_strings =
                        !is_character_complement && expression_may_contain_strings;
                    return;
                } else {
                    self.error(
                        diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                        self.pos,
                        1,
                    );
                }
            }
            _ => {
                if is_character_complement && self.may_contain_strings {
                    self.error(
                        diagnostics::ANYTHING_THAT_WOULD_POSSIBLY_MATCH_MORE_THAN_A_SINGLE_CHARACTER_IS_INVALID_INSIDE_A_NEGATED_CHARACTER_CLASS,
                        start,
                        self.pos - start,
                    );
                }
                expression_may_contain_strings = self.may_contain_strings;
            }
        }

        let mut operand = operand;
        while self.pos < self.body_end {
            let ch = self.char();
            match ch {
                '-' => {
                    self.inc_pos(1);
                    let ch2 = self.char();
                    if self.is_class_content_exit(ch2) {
                        self.may_contain_strings =
                            !is_character_complement && expression_may_contain_strings;
                        return;
                    }
                    if ch2 == '-' {
                        self.inc_pos(1);
                        self.error(
                            diagnostics::OPERATORS_MUST_NOT_BE_MIXED_WITHIN_A_CHARACTER_CLASS_WRAP_IT_IN_A_NESTED_CLASS_INSTEAD,
                            self.pos - 2,
                            2,
                        );
                        start = self.pos - 2;
                        operand = self.text[start..self.pos].to_string();
                        continue;
                    } else {
                        if operand.is_empty() {
                            self.error(
                                diagnostics::A_CHARACTER_CLASS_RANGE_MUST_NOT_BE_BOUNDED_BY_ANOTHER_CHARACTER_CLASS,
                                start,
                                self.pos - 1 - start,
                            );
                        }
                        let second_start = self.pos;
                        let second_operand = self.scan_class_set_operand();
                        if is_character_complement && self.may_contain_strings {
                            self.error(
                                diagnostics::ANYTHING_THAT_WOULD_POSSIBLY_MATCH_MORE_THAN_A_SINGLE_CHARACTER_IS_INVALID_INSIDE_A_NEGATED_CHARACTER_CLASS,
                                second_start,
                                self.pos - second_start,
                            );
                        }
                        expression_may_contain_strings =
                            expression_may_contain_strings || self.may_contain_strings;
                        if second_operand.is_empty() {
                            self.error(
                                diagnostics::A_CHARACTER_CLASS_RANGE_MUST_NOT_BE_BOUNDED_BY_ANOTHER_CHARACTER_CLASS,
                                second_start,
                                self.pos - second_start,
                            );
                        } else if !operand.is_empty() {
                            if let (Some((min_c, min_size)), Some((max_c, max_size))) = (
                                decode_first_rune(&operand),
                                decode_first_rune(&second_operand),
                            ) {
                                if operand.len() == min_size
                                    && second_operand.len() == max_size
                                    && (min_c as u32) > (max_c as u32)
                                {
                                    self.error(
                                        diagnostics::RANGE_OUT_OF_ORDER_IN_CHARACTER_CLASS,
                                        start,
                                        self.pos - start,
                                    );
                                }
                            }
                        }
                    }
                }
                '&' => {
                    start = self.pos;
                    self.inc_pos(1);
                    if self.char() == '&' {
                        self.inc_pos(1);
                        self.error(
                            diagnostics::OPERATORS_MUST_NOT_BE_MIXED_WITHIN_A_CHARACTER_CLASS_WRAP_IT_IN_A_NESTED_CLASS_INSTEAD,
                            self.pos - 2,
                            2,
                        );
                        if self.char() == '&' {
                            self.error(
                                diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                                self.pos,
                                1,
                            );
                            self.inc_pos(1);
                        }
                    } else {
                        self.error(
                            diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                            self.pos - 1,
                            1,
                        );
                    }
                    operand = self.text[start..self.pos].to_string();
                    continue;
                }
                _ => {}
            }
            if self.is_class_content_exit(self.char()) {
                break;
            }
            start = self.pos;
            let two = self.two_chars_at(self.pos);
            if matches!(two, Some([b'-', b'-']) | Some([b'&', b'&'])) {
                self.error(
                    diagnostics::OPERATORS_MUST_NOT_BE_MIXED_WITHIN_A_CHARACTER_CLASS_WRAP_IT_IN_A_NESTED_CLASS_INSTEAD,
                    self.pos,
                    2,
                );
                self.inc_pos(2);
                operand = self.text[start..self.pos].to_string();
            } else {
                operand = self.scan_class_set_operand();
            }
        }
        self.may_contain_strings = !is_character_complement && expression_may_contain_strings;
    }

    fn scan_class_set_sub_expression(&mut self, expression_type: ClassSetExpressionType) {
        let mut expression_may_contain_strings = self.may_contain_strings;
        while self.pos < self.body_end {
            let ch = self.char();
            if self.is_class_content_exit(ch) {
                break;
            }
            match ch {
                '-' => {
                    self.inc_pos(1);
                    if self.char() == '-' {
                        self.inc_pos(1);
                        if expression_type != ClassSetExpressionType::ClassSubtraction {
                            self.error(
                                diagnostics::OPERATORS_MUST_NOT_BE_MIXED_WITHIN_A_CHARACTER_CLASS_WRAP_IT_IN_A_NESTED_CLASS_INSTEAD,
                                self.pos - 2,
                                2,
                            );
                        }
                    } else {
                        self.error(
                            diagnostics::OPERATORS_MUST_NOT_BE_MIXED_WITHIN_A_CHARACTER_CLASS_WRAP_IT_IN_A_NESTED_CLASS_INSTEAD,
                            self.pos - 1,
                            1,
                        );
                    }
                }
                '&' => {
                    self.inc_pos(1);
                    if self.char() == '&' {
                        self.inc_pos(1);
                        if expression_type != ClassSetExpressionType::ClassIntersection {
                            self.error(
                                diagnostics::OPERATORS_MUST_NOT_BE_MIXED_WITHIN_A_CHARACTER_CLASS_WRAP_IT_IN_A_NESTED_CLASS_INSTEAD,
                                self.pos - 2,
                                2,
                            );
                        }
                        if self.char() == '&' {
                            self.error(
                                diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                                self.pos,
                                1,
                            );
                            self.inc_pos(1);
                        }
                    } else {
                        self.error(
                            diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                            self.pos - 1,
                            1,
                        );
                    }
                }
                _ => {
                    if expression_type == ClassSetExpressionType::ClassSubtraction
                        || expression_type == ClassSetExpressionType::ClassIntersection
                    {
                        self.error(diagnostics::X_0_EXPECTED, self.pos, 0);
                    }
                }
            }
            let ch2 = self.char();
            if self.is_class_content_exit(ch2) {
                self.error(diagnostics::EXPECTED_A_CLASS_SET_OPERAND, self.pos, 0);
                break;
            }
            self.scan_class_set_operand();
            if expression_type == ClassSetExpressionType::ClassIntersection {
                expression_may_contain_strings =
                    expression_may_contain_strings && self.may_contain_strings;
            }
        }
        self.may_contain_strings = expression_may_contain_strings;
    }

    // ClassSetOperand ::=
    //   | '[' ClassSetExpression ']'
    //   | '\' CharacterClassEscape
    //   | '\q{' ClassStringDisjunctionContents '}'
    //   | ClassSetCharacter
    fn scan_class_set_operand(&mut self) -> String {
        self.may_contain_strings = false;
        let ch = self.char();
        match ch {
            '[' => {
                self.inc_pos(1);
                self.scan_class_set_expression();
                self.scan_expected_char(']');
                String::new()
            }
            '\\' => {
                self.inc_pos(1);
                if self.scan_character_class_escape() {
                    return String::new();
                } else if self.char() == 'q' {
                    self.inc_pos(1);
                    if self.char() == '{' {
                        self.inc_pos(1);
                        self.scan_class_string_disjunction_contents();
                        self.scan_expected_char('}');
                        return String::new();
                    } else {
                        self.error(
                            diagnostics::X_Q_MUST_BE_FOLLOWED_BY_STRING_ALTERNATIVES_ENCLOSED_IN_BRACES,
                            self.pos - 2,
                            2,
                        );
                        return "q".to_string();
                    }
                }
                self.inc_pos(-1);
                // Fallthrough to ClassSetCharacter.
                self.scan_class_set_character()
            }
            _ => self.scan_class_set_character(),
        }
    }

    // ClassStringDisjunctionContents ::= ClassSetCharacter* ('|' ClassSetCharacter*)*
    fn scan_class_string_disjunction_contents(&mut self) {
        // Precondition: pos is at the char following '{'.
        let mut character_count = 0;
        while self.pos < self.body_end {
            let ch = self.char();
            match ch {
                '}' => {
                    if character_count != 1 {
                        self.may_contain_strings = true;
                    }
                    return;
                }
                '|' => {
                    if character_count != 1 {
                        self.may_contain_strings = true;
                    }
                    self.inc_pos(1);
                    character_count = 0;
                }
                _ => {
                    self.scan_class_set_character();
                    character_count += 1;
                }
            }
        }
    }

    // ClassSetCharacter ::=
    //   | SourceCharacter -- ClassSetSyntaxCharacter -- ClassSetReservedDoublePunctuator
    //   | '\' (CharacterEscape | ClassSetReservedPunctuator | 'b')
    fn scan_class_set_character(&mut self) -> String {
        let ch = self.char();
        if ch == '\\' {
            self.inc_pos(1);
            let inner_ch = self.char();
            match inner_ch {
                'b' => {
                    self.inc_pos(1);
                    return "\u{0008}".to_string();
                }
                '&' | '-' | '!' | '#' | '%' | ',' | ':' | ';' | '<' | '=' | '>' | '@' | '`'
                | '~' => {
                    self.inc_pos(1);
                    return inner_ch.to_string();
                }
                _ => {
                    return self.scan_character_escape(false);
                }
            }
        } else if self.pos + 1 < self.body_end && ch == self.char_at(1) {
            match ch {
                '&' | '!' | '#' | '%' | '*' | '+' | ',' | '.' | ':' | ';' | '<' | '=' | '>'
                | '?' | '@' | '`' | '~' => {
                    self.error(
                        diagnostics::A_CHARACTER_CLASS_MUST_NOT_CONTAIN_A_RESERVED_DOUBLE_PUNCTUATOR_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                        self.pos,
                        2,
                    );
                    self.inc_pos(2);
                    return self.text[self.pos - 2..self.pos].to_string();
                }
                _ => {}
            }
        }
        match ch {
            '/' | '(' | ')' | '[' | ']' | '{' | '}' | '-' | '|' => {
                self.error(
                    diagnostics::UNEXPECTED_0_DID_YOU_MEAN_TO_ESCAPE_IT_WITH_BACKSLASH,
                    self.pos,
                    1,
                );
                self.inc_pos(1);
                ch.to_string()
            }
            _ => self.scan_source_character(),
        }
    }

    // ClassAtom ::=
    //   | SourceCharacter but not one of '\' or ']'
    //   | '\' ClassEscape
    fn scan_class_atom(&mut self) -> String {
        let ch = self.char();
        if ch == '\\' {
            self.inc_pos(1);
            let ch2 = self.char();
            match ch2 {
                'b' => {
                    self.inc_pos(1);
                    "\u{0008}".to_string()
                }
                '-' => {
                    self.inc_pos(1);
                    ch2.to_string()
                }
                _ => {
                    if self.scan_character_class_escape() {
                        return String::new();
                    }
                    self.scan_character_escape(false)
                }
            }
        } else {
            self.scan_source_character()
        }
    }

    // CharacterClassEscape ::=
    //   | 'd' | 'D' | 's' | 'S' | 'w' | 'W'
    //   | [+AnyUnicodeMode] ('P' | 'p') '{' UnicodePropertyValueExpression '}'
    fn scan_character_class_escape(&mut self) -> bool {
        // Precondition: pos is at the char following a backslash.
        let mut is_character_complement = false;
        let start = self.pos - 1; // backslash position
        let ch = self.char();
        match ch {
            'd' | 'D' | 's' | 'S' | 'w' | 'W' => {
                self.inc_pos(1);
                return true;
            }
            'P' => {
                is_character_complement = true;
                // Fall through to the 'p' / 'P' handling below.
            }
            'p' => {
                // Fall through to the 'p' / 'P' handling below.
            }
            _ => return false,
        }

        self.inc_pos(1); // consume 'p' / 'P'
        if self.char() == '{' {
            self.inc_pos(1); // consume '{'
            let property_name_or_value_start = self.pos;
            let property_name_or_value = self.scan_word_characters();
            if self.char() == '=' {
                // `name=value` form.
                let property_name =
                    unicode_properties::non_binary_property_canonical(&property_name_or_value)
                        .unwrap_or("");
                if self.pos == property_name_or_value_start {
                    self.error(diagnostics::EXPECTED_A_UNICODE_PROPERTY_NAME, self.pos, 0);
                } else if property_name.is_empty() {
                    self.error(
                        diagnostics::UNKNOWN_UNICODE_PROPERTY_NAME,
                        property_name_or_value_start,
                        self.pos - property_name_or_value_start,
                    );
                    // Spelling suggestion skipped.
                }
                self.inc_pos(1); // consume '='
                let property_value_start = self.pos;
                let property_value = self.scan_word_characters();
                if self.pos == property_value_start {
                    self.error(diagnostics::EXPECTED_A_UNICODE_PROPERTY_VALUE, self.pos, 0);
                } else if !property_name.is_empty() {
                    if !unicode_properties::is_valid_unicode_property_value(
                        property_name,
                        &property_value,
                    ) {
                        self.error(
                            diagnostics::UNKNOWN_UNICODE_PROPERTY_VALUE,
                            property_value_start,
                            self.pos - property_value_start,
                        );
                        // Spelling suggestion skipped.
                    }
                }
            } else {
                // Lone property name or value.
                if self.pos == property_name_or_value_start {
                    self.error(
                        diagnostics::EXPECTED_A_UNICODE_PROPERTY_NAME_OR_VALUE,
                        self.pos,
                        0,
                    );
                } else if unicode_properties::is_binary_unicode_property_of_strings(
                    &property_name_or_value,
                ) {
                    if !self.unicode_sets_mode {
                        self.error(
                            diagnostics::ANY_UNICODE_PROPERTY_THAT_WOULD_POSSIBLY_MATCH_MORE_THAN_A_SINGLE_CHARACTER_IS_ONLY_AVAILABLE_WHEN_THE_UNICODE_SETS_V_FLAG_IS_SET,
                            property_name_or_value_start,
                            self.pos - property_name_or_value_start,
                        );
                    } else if is_character_complement {
                        self.error(
                            diagnostics::ANYTHING_THAT_WOULD_POSSIBLY_MATCH_MORE_THAN_A_SINGLE_CHARACTER_IS_INVALID_INSIDE_A_NEGATED_CHARACTER_CLASS,
                            property_name_or_value_start,
                            self.pos - property_name_or_value_start,
                        );
                    } else {
                        self.may_contain_strings = true;
                    }
                } else if !unicode_properties::is_valid_unicode_property_value(
                    "General_Category",
                    &property_name_or_value,
                ) && !unicode_properties::is_binary_unicode_property(
                    &property_name_or_value,
                ) {
                    self.error(
                        diagnostics::UNKNOWN_UNICODE_PROPERTY_NAME_OR_VALUE,
                        property_name_or_value_start,
                        self.pos - property_name_or_value_start,
                    );
                    // Spelling suggestion skipped.
                }
            }
            self.scan_expected_char('}');
            if !self.any_unicode_mode {
                self.error(
                    diagnostics::UNICODE_PROPERTY_VALUE_EXPRESSIONS_ARE_ONLY_AVAILABLE_WHEN_THE_UNICODE_U_FLAG_OR_THE_UNICODE_SETS_V_FLAG_IS_SET,
                    start,
                    self.pos - start,
                );
            }
        } else if self.any_unicode_mode_or_non_annex_b {
            self.error(
                diagnostics::X_0_MUST_BE_FOLLOWED_BY_A_UNICODE_PROPERTY_VALUE_EXPRESSION_ENCLOSED_IN_BRACES,
                self.pos - 2,
                2,
            );
        } else {
            self.inc_pos(-1); // back up so the caller can re-scan 'p'/'P'
            return false;
        }
        true
    }

    // ────────────────────────────────────────────────────────────────────
    // Source characters
    // ────────────────────────────────────────────────────────────────────

    /// Decode one UTF-8 rune and advance. Simplified from Go's
    /// `scanSourceCharacter`: the non-unicode-mode surrogate-splitting
    /// bookkeeping (`pendingLowSurrogate`) is skipped, as it is not needed for
    /// validation.
    fn scan_source_character(&mut self) -> String {
        if self.pos >= self.body_end {
            return String::new();
        }
        let (c, size) = decode_rune_at(self.text, self.pos);
        self.pos += size;
        c.to_string()
    }
}
