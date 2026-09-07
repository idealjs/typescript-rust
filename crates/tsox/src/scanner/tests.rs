use super::*;
use std::sync::{Mutex, OnceLock};

static RECORDED_ERRORS: OnceLock<Mutex<Vec<(DiagnosticKind, usize, usize)>>> = OnceLock::new();

fn recorded_errors() -> &'static Mutex<Vec<(DiagnosticKind, usize, usize)>> {
    RECORDED_ERRORS.get_or_init(|| Mutex::new(Vec::new()))
}

fn record_error(kind: DiagnosticKind, start: usize, length: usize) {
    recorded_errors()
        .lock()
        .unwrap()
        .push((kind, start, length));
}

#[test]
fn scan_identifiers_and_keywords() {
    let mut s = Scanner::new("foo const let");
    assert_eq!(s.scan(), SyntaxKind::Identifier);
    assert_eq!(s.token_text(), "foo");
    assert_eq!(s.scan(), SyntaxKind::ConstKeyword);
    assert_eq!(s.token_text(), "const");
    assert_eq!(s.scan(), SyntaxKind::LetKeyword);
    assert_eq!(s.token_text(), "let");
    assert_eq!(s.scan(), SyntaxKind::EndOfFile);
}

#[test]
fn scan_private_identifier() {
    let mut s = Scanner::new("#name = 1");
    assert_eq!(s.scan(), SyntaxKind::PrivateIdentifier);
    assert_eq!(s.token_text(), "#name");
    assert_eq!(s.scan(), SyntaxKind::EqualsToken);
    assert_eq!(s.scan(), SyntaxKind::NumericLiteral);
    assert_eq!(s.scan(), SyntaxKind::EndOfFile);
}

#[test]
fn scan_shebang_at_file_start_is_trivia() {
    let mut s = Scanner::new("#!/usr/bin/env node\nlet x = 1;");
    assert_eq!(s.scan(), SyntaxKind::LetKeyword);
    assert_eq!(s.scan(), SyntaxKind::Identifier);
    assert_eq!(s.token_text(), "x");
}

#[test]
fn scan_numbers() {
    let mut s = Scanner::new("42 3.14 0x1F 0b101 0o77 100n");
    assert_eq!(s.scan(), SyntaxKind::NumericLiteral);
    assert_eq!(s.token_text(), "42");
    assert_eq!(s.scan(), SyntaxKind::NumericLiteral);
    assert_eq!(s.token_text(), "3.14");
    assert_eq!(s.scan(), SyntaxKind::NumericLiteral);
    assert_eq!(s.token_text(), "0x1F");
    assert_eq!(s.scan(), SyntaxKind::NumericLiteral);
    assert_eq!(s.token_text(), "0b101");
    assert_eq!(s.scan(), SyntaxKind::NumericLiteral);
    assert_eq!(s.token_text(), "0o77");
    assert_eq!(s.scan(), SyntaxKind::BigIntLiteral);
    assert_eq!(s.token_text(), "100n");
}

#[test]
fn scan_strings() {
    let mut s = Scanner::new("\"hello\" 'world'");
    assert_eq!(s.scan(), SyntaxKind::StringLiteral);
    assert_eq!(s.token_text(), "\"hello\"");
    assert_eq!(s.scan(), SyntaxKind::StringLiteral);
    assert_eq!(s.token_text(), "'world'");
}

#[test]
fn scan_string_escape_sequences() {
    let mut s = Scanner::new(r#""\x22""#);
    assert_eq!(s.scan(), SyntaxKind::StringLiteral);
    assert_eq!(s.token_text(), r#""\x22""#);
    assert_eq!(s.token_value(), "\"");

    let mut s = Scanner::new(r#""\u{1F600}""#);
    assert_eq!(s.scan(), SyntaxKind::StringLiteral);
    assert_eq!(s.token_value(), "\u{1F600}");

    let mut s = Scanner::new(r#""\u0041""#);
    assert_eq!(s.scan(), SyntaxKind::StringLiteral);
    assert_eq!(s.token_value(), "A");

    let mut s = Scanner::new("\"hello\\\nworld\"");
    assert_eq!(s.scan(), SyntaxKind::StringLiteral);
    assert_eq!(s.token_value(), "helloworld");

    let mut s = Scanner::new(r#""\\\"""#);
    assert_eq!(s.scan(), SyntaxKind::StringLiteral);
    assert_eq!(s.token_value(), "\\\"");
}

#[test]
fn scan_punctuation() {
    let mut s = Scanner::new("=> === ... ??=");
    assert_eq!(s.scan(), SyntaxKind::EqualsGreaterThanToken);
    assert_eq!(s.scan(), SyntaxKind::EqualsEqualsEqualsToken);
    assert_eq!(s.scan(), SyntaxKind::DotDotDotToken);
    assert_eq!(s.scan(), SyntaxKind::QuestionQuestionEqualsToken);
}

#[test]
fn scan_non_ascii_unknown_characters_do_not_split_utf8() {
    recorded_errors().lock().unwrap().clear();
    let mut s = Scanner::new("· 中 🦀").with_error_callback(record_error);

    assert_eq!(s.scan(), SyntaxKind::Unknown);
    assert_eq!(s.token_text(), "·");

    assert_eq!(s.scan(), SyntaxKind::Identifier);
    assert_eq!(s.token_text(), "中");

    assert_eq!(s.scan(), SyntaxKind::Unknown);
    assert_eq!(s.token_text(), "🦀");

    assert_eq!(s.scan(), SyntaxKind::EndOfFile);

    let errors = recorded_errors().lock().unwrap();
    assert_eq!(
        errors.as_slice(),
        &[
            (DiagnosticKind::InvalidCharacter, 0, "·".len()),
            (DiagnosticKind::InvalidCharacter, "· 中 ".len(), "🦀".len()),
        ]
    );
}

#[test]
fn scan_comments() {
    let mut s = Scanner::new("// comment\nfoo /* block */ bar");
    assert_eq!(s.scan(), SyntaxKind::Identifier);
    assert_eq!(s.token_text(), "foo");
    assert!(s.has_preceding_line_break());
    assert_eq!(s.scan(), SyntaxKind::Identifier);
    assert_eq!(s.token_text(), "bar");
}

#[test]
fn scan_template_literal() {
    let mut s = Scanner::new("`hello`");
    assert_eq!(s.scan(), SyntaxKind::NoSubstitutionTemplateLiteral);
    assert_eq!(s.token_text(), "`hello`");

    let mut s = Scanner::new("`hello ${");
    assert_eq!(s.scan(), SyntaxKind::TemplateHead);
}

#[test]
fn keyword_lookup() {
    assert_eq!(string_to_keyword("class"), Some(SyntaxKind::ClassKeyword));
    assert_eq!(string_to_keyword("foobar"), None);
}

#[test]
fn re_scan_slash_token_basic_regex() {
    let mut s = Scanner::new("/foo/g");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::SlashToken);
    s.re_scan_slash_token();
    assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
    assert_eq!(s.token_text(), "/foo/g");
}

#[test]
fn re_scan_slash_token_regex_with_flags() {
    let mut s = Scanner::new("/pattern/gim");
    s.scan();
    s.re_scan_slash_token();
    assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
    assert_eq!(s.token_text(), "/pattern/gim");
}

#[test]
fn re_scan_slash_token_regex_with_char_class() {
    let mut s = Scanner::new(r"/[\/]/");
    s.scan();
    s.re_scan_slash_token();
    assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
    assert_eq!(s.token_text(), r"/[\/]/");
}

#[test]
fn re_scan_slash_token_regex_with_escape() {
    let mut s = Scanner::new(r"/a\/b/");
    s.scan();
    s.re_scan_slash_token();
    assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
    assert_eq!(s.token_text(), r"/a\/b/");
}

#[test]
fn re_scan_slash_token_slash_equals() {
    let mut s = Scanner::new("/=/");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::SlashEqualsToken);
    s.re_scan_slash_token();
    assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
    assert_eq!(s.token_text(), "/=/");
}

#[test]
fn re_scan_slash_token_unterminated() {
    let mut s = Scanner::new("/foo");
    s.scan();
    s.re_scan_slash_token();
    assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
    let errors = s.take_errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].kind,
        DiagnosticKind::UnterminatedRegularExpression
    );
}

#[test]
fn re_scan_slash_token_unterminated_newline() {
    let mut s = Scanner::new("/foo\nbar");
    s.scan();
    s.re_scan_slash_token();
    assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
    let errors = s.take_errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].kind,
        DiagnosticKind::UnterminatedRegularExpression
    );
}

#[test]
fn re_scan_slash_token_valid_flags_no_errors() {
    let mut s = Scanner::new("/pattern/dgimsy");
    s.scan();
    s.re_scan_slash_token();
    assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
    assert_eq!(s.token_text(), "/pattern/dgimsy");
    assert!(s.take_errors().is_empty());
}

#[test]
fn re_scan_slash_token_unknown_flag_reports_ts1499() {
    let mut s = Scanner::new("/foo/zz");
    s.scan();
    s.re_scan_slash_token();
    assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
    assert_eq!(s.token_text(), "/foo/zz");
    let errors = s.take_errors();
    assert_eq!(errors.len(), 2, "expected two TS1499 errors for 'zz'");
    assert_eq!(errors[0].kind, DiagnosticKind::UnknownRegularExpressionFlag);
    assert_eq!(errors[0].pos, "/foo/".len());
    assert_eq!(errors[0].length, 1);
    assert_eq!(errors[1].kind, DiagnosticKind::UnknownRegularExpressionFlag);
    assert_eq!(errors[1].pos, "/foo/z".len());
}

#[test]
fn re_scan_slash_token_duplicate_flag_reports_ts1500() {
    let mut s = Scanner::new("/foo/gg");
    s.scan();
    s.re_scan_slash_token();
    assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
    assert_eq!(s.token_text(), "/foo/gg");
    let errors = s.take_errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].kind,
        DiagnosticKind::DuplicateRegularExpressionFlag
    );
    assert_eq!(errors[0].pos, "/foo/g".len());
    assert_eq!(errors[0].length, 1);
}

#[test]
fn re_scan_slash_token_u_and_v_mutually_exclusive_reports_ts1502() {
    let mut s = Scanner::new("/foo/uv");
    s.scan();
    s.re_scan_slash_token();
    assert_eq!(s.token(), SyntaxKind::RegularExpressionLiteral);
    let errors = s.take_errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].kind,
        DiagnosticKind::UnicodeUAndVFlagsMutuallyExclusive
    );
    assert_eq!(errors[0].pos, "/foo/u".len());
    assert_eq!(errors[0].length, 1);

    let mut s = Scanner::new("/foo/vu");
    s.scan();
    s.re_scan_slash_token();
    let errors = s.take_errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].kind,
        DiagnosticKind::UnicodeUAndVFlagsMutuallyExclusive
    );
    assert_eq!(errors[0].pos, "/foo/v".len());
}

#[test]
fn re_scan_slash_token_mixed_flag_errors() {
    let mut s = Scanner::new("/foo/guz");
    s.scan();
    s.re_scan_slash_token();
    let errors = s.take_errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, DiagnosticKind::UnknownRegularExpressionFlag);
    assert_eq!(errors[0].pos, "/foo/gu".len());
}

#[test]
fn comment_directive_ts_expect_error_single_line() {
    let mut s = Scanner::new("// @ts-expect-error\n");
    s.scan();
    let directives = s.comment_directives();
    assert_eq!(directives.len(), 1);
    assert_eq!(directives[0].kind, CommentDirectiveKind::ExpectError);
    assert_eq!(directives[0].pos, 0);
    assert_eq!(directives[0].end, 19);
}

#[test]
fn comment_directive_ts_ignore_single_line() {
    let mut s = Scanner::new("// @ts-ignore");
    s.scan();
    let directives = s.comment_directives();
    assert_eq!(directives.len(), 1);
    assert_eq!(directives[0].kind, CommentDirectiveKind::Ignore);
}

#[test]
fn comment_directive_triple_slash_ts_ignore() {
    let mut s = Scanner::new("/// @ts-ignore");
    s.scan();
    let directives = s.comment_directives();
    assert_eq!(directives.len(), 1);
    assert_eq!(directives[0].kind, CommentDirectiveKind::Ignore);
}

#[test]
fn comment_directive_multiline_ts_expect_error() {
    let mut s = Scanner::new("/* @ts-expect-error */");
    s.scan();
    let directives = s.comment_directives();
    assert_eq!(directives.len(), 1);
    assert_eq!(directives[0].kind, CommentDirectiveKind::ExpectError);
}

#[test]
fn comment_directive_no_directive_for_regular_comment() {
    let mut s = Scanner::new("// just a regular comment");
    s.scan();
    assert!(s.comment_directives().is_empty());

    let mut s = Scanner::new("/* block comment */");
    s.scan();
    assert!(s.comment_directives().is_empty());
}

#[test]
fn comment_directive_multiple_in_source() {
    let mut s = Scanner::new("// @ts-ignore\nlet x = 1;\n// @ts-expect-error\n");
    while s.scan() != SyntaxKind::EndOfFile {}
    let directives = s.comment_directives();
    assert_eq!(directives.len(), 2);
    assert_eq!(directives[0].kind, CommentDirectiveKind::Ignore);
    assert_eq!(directives[1].kind, CommentDirectiveKind::ExpectError);
}

#[test]
fn skip_trivia_whitespace_and_newlines() {
    assert_eq!(skip_trivia("  \t\n  x", 0), 6);
    assert_eq!(skip_trivia("\n\n\nx", 0), 3);
    assert_eq!(skip_trivia("x", 0), 0);

    assert_eq!(skip_trivia("", 0), 0);
}

#[test]
fn skip_trivia_single_line_comment() {
    assert_eq!(skip_trivia("// comment\nx", 0), 11);

    assert_eq!(skip_trivia("// eof", 0), 6);
}

#[test]
fn skip_trivia_multi_line_comment() {
    assert_eq!(skip_trivia("/* comment */x", 0), 13);

    assert_eq!(skip_trivia("/* unterminated", 0), 15);

    assert_eq!(skip_trivia("abc", 0), 0);
}

#[test]
fn skip_trivia_shebang_at_start() {
    assert_eq!(skip_trivia("#!/usr/bin/env node\nlet x;", 0), 20);

    assert_eq!(skip_trivia(" #!/foo", 1), 1);
}

#[test]
fn skip_trivia_combined() {
    assert_eq!(
        skip_trivia("#!/usr/bin/env node\n// hello\n/* world */\nlet x;", 0),
        41
    );
}

#[test]
fn get_shebang_returns_text() {
    assert_eq!(
        get_shebang("#!/usr/bin/env node\nlet x;"),
        "#!/usr/bin/env node"
    );
    assert_eq!(get_shebang("let x;"), "");
    assert_eq!(get_shebang("#!only\nmore"), "#!only");
}

#[test]
fn full_start_pos_tracks_leading_trivia() {
    let mut s = Scanner::new("let x = 1;");
    s.scan();
    assert_eq!(s.full_start_pos(), 0);
    assert_eq!(s.token_pos(), 0);
    assert_eq!(s.token(), SyntaxKind::LetKeyword);

    s.scan();
    assert_eq!(s.full_start_pos(), 3);
    assert_eq!(s.token_pos(), 4);
    assert_eq!(s.token(), SyntaxKind::Identifier);

    s.scan();
    assert_eq!(s.full_start_pos(), 5);
    assert_eq!(s.token_pos(), 6);
    assert_eq!(s.token(), SyntaxKind::EqualsToken);
}

#[test]
fn full_start_pos_preserved_across_comments() {
    let mut s = Scanner::new("// hi\nlet x;");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::LetKeyword);
    assert_eq!(s.full_start_pos(), 0);
    assert_eq!(s.token_pos(), 6);

    let mut s = Scanner::new("a /* c */ b");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::Identifier);
    assert_eq!(s.token_pos(), 0);
    s.scan();
    assert_eq!(s.token(), SyntaxKind::Identifier);

    assert_eq!(s.full_start_pos(), 1);
    assert_eq!(s.token_pos(), 10);
}

#[test]
fn get_leading_comment_ranges_basic() {
    let text = "// first\n// second\nlet x;";
    let ranges = get_leading_comment_ranges(text, 0);
    assert_eq!(ranges.len(), 2);
    assert_eq!(ranges[0].kind, CommentRangeKind::SingleLine);
    assert_eq!(ranges[0].pos, 0);
    assert_eq!(ranges[0].end, 8);
    assert!(ranges[0].has_trailing_new_line);
    assert_eq!(ranges[1].pos, 9);
    assert_eq!(ranges[1].end, 18);
    assert!(ranges[1].has_trailing_new_line);
}

#[test]
fn get_leading_comment_ranges_multi_line() {
    let text = "/* hello */let x;";
    let ranges = get_leading_comment_ranges(text, 0);
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0].kind, CommentRangeKind::MultiLine);
    assert_eq!(ranges[0].pos, 0);
    assert_eq!(ranges[0].end, 11);
    assert!(!ranges[0].has_trailing_new_line);
}

#[test]
fn get_leading_comment_ranges_from_middle() {
    let text = "let x; // trailing\n// leading for next\nlet y;";

    let ranges = get_leading_comment_ranges(text, 18);
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0].kind, CommentRangeKind::SingleLine);
    assert_eq!(ranges[0].pos, 19);
    assert_eq!(ranges[0].end, 38);
    assert!(ranges[0].has_trailing_new_line);
}

#[test]
fn get_leading_comment_ranges_none() {
    let ranges = get_leading_comment_ranges("let x;", 0);
    assert!(ranges.is_empty());
}

#[test]
fn get_trailing_comment_ranges_basic() {
    let text = "let x; // trailing\nlet y;";
    let ranges = get_trailing_comment_ranges(text, 6);
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0].kind, CommentRangeKind::SingleLine);
    assert_eq!(ranges[0].pos, 7);
    assert_eq!(ranges[0].end, 18);
    assert!(ranges[0].has_trailing_new_line);
}

#[test]
fn get_trailing_comment_ranges_stops_at_line_break() {
    let text = "let x;\nlet y; // c\n";
    let ranges = get_trailing_comment_ranges(text, 0);
    assert!(ranges.is_empty());
}

#[test]
fn get_trailing_comment_ranges_multi_line() {
    let text = "let x; /* c */ let y;";
    let ranges = get_trailing_comment_ranges(text, 6);
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0].kind, CommentRangeKind::MultiLine);
    assert_eq!(ranges[0].pos, 7);
    assert_eq!(ranges[0].end, 14);
    assert!(!ranges[0].has_trailing_new_line);
}

#[test]
fn get_leading_comment_ranges_shebang_skipped() {
    let text = "#!/usr/bin/env node\n// real comment\nlet x;";
    let ranges = get_leading_comment_ranges(text, 0);
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0].pos, 20);
    assert_eq!(ranges[0].end, 35);
    assert_eq!(ranges[0].kind, CommentRangeKind::SingleLine);
    assert!(ranges[0].has_trailing_new_line);
}

#[test]
fn token_flags_preceding_line_break_set() {
    let mut s = Scanner::new("foo\nbar");
    s.scan();
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_PRECEDING_LINE_BREAK
    ));
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_PRECEDING_LINE_BREAK
    ));
    assert!(s.has_preceding_line_break());
}

#[test]
fn token_flags_single_quote_string() {
    let mut s = Scanner::new("'abc'");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::StringLiteral);
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_SINGLE_QUOTE
    ));

    let mut s = Scanner::new("\"abc\"");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::StringLiteral);
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_SINGLE_QUOTE
    ));
}

#[test]
fn token_flags_unterminated_string() {
    let mut s = Scanner::new("'abc\ndef'");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::StringLiteral);
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_UNTERMINATED
    ));
}

#[test]
fn token_flags_terminated_string_no_unterminated() {
    let mut s = Scanner::new("'abc'");
    s.scan();
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_UNTERMINATED
    ));
}

#[test]
fn token_flags_hex_numeric_literal() {
    let mut s = Scanner::new("0x1F");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::NumericLiteral);
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_HEX_SPECIFIER
    ));

    assert!(token_flags_intersects(
        s.token_flags(),
        TOKEN_FLAGS_WITH_SPECIFIER
    ));
}

#[test]
fn token_flags_binary_numeric_literal() {
    let mut s = Scanner::new("0b1010");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::NumericLiteral);
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_BINARY_SPECIFIER
    ));
}

#[test]
fn token_flags_octal_numeric_literal() {
    let mut s = Scanner::new("0o777");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::NumericLiteral);
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_OCTAL_SPECIFIER
    ));
}

#[test]
fn token_flags_scientific_numeric_literal() {
    let mut s = Scanner::new("10e2");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::NumericLiteral);
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_SCIENTIFIC
    ));
}

#[test]
fn token_flags_contains_leading_zero() {
    let mut s = Scanner::new("0888");
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_LEADING_ZERO
    ));
    let mut s = Scanner::new("0x1F");
    s.scan();
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_LEADING_ZERO
    ));
}

#[test]
fn token_flags_plain_decimal_none() {
    let mut s = Scanner::new("123");
    s.scan();
    let flags = s.token_flags();
    assert_eq!(flags & TOKEN_FLAGS_NUMERIC_LITERAL_FLAGS, TOKEN_FLAGS_NONE);
}

#[test]
fn token_flags_reset_between_tokens() {
    let mut s = Scanner::new("0x1F 'str'");
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_HEX_SPECIFIER
    ));
    s.scan();
    assert_eq!(s.token(), SyntaxKind::StringLiteral);
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_HEX_SPECIFIER
    ));
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_SINGLE_QUOTE
    ));
}

#[test]
fn token_flags_unterminated_template() {
    let mut s = Scanner::new("`abc");
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_UNTERMINATED
    ));
}

#[test]
fn skip_trivia_ex_stop_after_line_break() {
    let text = "  \n  x";
    let opts = SkipTriviaOptions {
        stop_after_line_break: true,
        ..Default::default()
    };
    assert_eq!(skip_trivia_ex(text, 0, &opts, None), 3);

    assert_eq!(skip_trivia(text, 0), 5);
}

#[test]
fn skip_trivia_ex_stop_at_comments() {
    let text = "  // c\nx";
    let opts = SkipTriviaOptions {
        stop_at_comments: true,
        ..Default::default()
    };

    assert_eq!(skip_trivia_ex(text, 0, &opts, None), 2);

    assert_eq!(skip_trivia(text, 0), 7);
}

#[test]
fn skip_trivia_ex_in_jsdoc_consumes_leading_asterisk() {
    let text = "\n * @param";
    let opts = SkipTriviaOptions {
        in_jsdoc: true,
        ..Default::default()
    };
    assert_eq!(skip_trivia_ex(text, 0, &opts, None), 4);

    assert_eq!(skip_trivia(text, 0), 2);
}

#[test]
fn skip_trivia_ex_jsdoc_star_only_after_line_break() {
    let text = " * foo";
    let opts = SkipTriviaOptions {
        in_jsdoc: true,
        ..Default::default()
    };

    assert_eq!(skip_trivia_ex(text, 0, &opts, None), 1);
}

#[test]
fn is_conflict_marker_trivia_detects_markers() {
    assert!(is_conflict_marker_trivia("<<<<<<< head\n", 0));

    assert!(is_conflict_marker_trivia("x\n>>>>>>> branch\n", 2));

    assert!(is_conflict_marker_trivia("x\n=======\n", 2));

    assert!(is_conflict_marker_trivia("x\n||||||| base\n", 2));

    assert!(!is_conflict_marker_trivia("<<<<<< \n", 0));

    assert!(!is_conflict_marker_trivia("<<<<<<<x\n", 0));

    assert!(!is_conflict_marker_trivia("x\n|||||||\n", 2));

    assert!(!is_conflict_marker_trivia("a <<<<<<< \n", 2));

    assert!(!is_conflict_marker_trivia("<x\n", 0));
}

#[test]
fn skip_trivia_ex_consumes_conflict_marker() {
    let text = "<<<<<<< a\nshared\n=======\n>>>>>>> b\nx";
    let pos = skip_trivia_ex(text, 0, &SkipTriviaOptions::default(), None);
    assert_eq!(&text[pos..], "shared\n=======\n>>>>>>> b\nx");
}

#[test]
fn skip_trivia_ex_reports_conflict_marker_error() {
    use std::cell::RefCell;
    let text = "<<<<<<< a\nx";
    let reported: RefCell<Vec<(usize, usize)>> = RefCell::new(Vec::new());
    let opts = SkipTriviaOptions::default();
    skip_trivia_ex(
        text,
        0,
        &opts,
        Some(&|p, l| reported.borrow_mut().push((p, l))),
    );
    assert_eq!(
        reported.borrow().as_slice(),
        &[(0, MERGE_CONFLICT_MARKER_LENGTH)]
    );
}

#[test]
fn skip_trivia_ex_pipe_divider_marker() {
    let text = "<<<<<<< a\nlocal\n||||||| base\nshared\n=======\nremote\n>>>>>>> b\nx";
    let pos = skip_trivia_ex(text, 0, &SkipTriviaOptions::default(), None);
    assert_eq!(
        &text[pos..],
        "local\n||||||| base\nshared\n=======\nremote\n>>>>>>> b\nx"
    );
}

#[test]
fn token_flags_preceding_jsdoc_comment() {
    let mut s = Scanner::new("/** doc */\nlet x");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::LetKeyword);
    assert!(s.has_preceding_jsdoc_comment());
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_PRECEDING_JSDOC_COMMENT
    ));
}

#[test]
fn token_flags_non_jsdoc_multi_line_comment() {
    let mut s = Scanner::new("/* not jsdoc */\nlet x");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::LetKeyword);
    assert!(!s.has_preceding_jsdoc_comment());
}

#[test]
fn token_flags_empty_jsdoc_comment_not_flagged() {
    let mut s = Scanner::new("/**/\nlet x");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::LetKeyword);
    assert!(!s.has_preceding_jsdoc_comment());
}

#[test]
fn token_flags_jsdoc_deprecated_tag() {
    let mut s = Scanner::new("/**\n * @deprecated\n */\nlet x");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::LetKeyword);
    assert!(s.has_preceding_jsdoc_comment());
    assert!(s.has_preceding_jsdoc_with_deprecated_tag());
    assert!(!s.has_preceding_jsdoc_with_see_or_link());
}

#[test]
fn token_flags_jsdoc_see_tag() {
    let mut s = Scanner::new("/**\n * @see foo\n */\nlet x");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::LetKeyword);
    assert!(s.has_preceding_jsdoc_with_see_or_link());
    assert!(!s.has_preceding_jsdoc_with_deprecated_tag());
}

#[test]
fn token_flags_jsdoc_link_tag() {
    let mut s = Scanner::new("/**\n * {@link foo}\n */\nlet x");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::LetKeyword);
    assert!(s.has_preceding_jsdoc_with_see_or_link());
}

#[test]
fn token_flags_jsdoc_both_tags() {
    let mut s = Scanner::new("/**\n * @deprecated\n * @see foo\n */\nlet x");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::LetKeyword);
    assert!(s.has_preceding_jsdoc_with_deprecated_tag());
    assert!(s.has_preceding_jsdoc_with_see_or_link());
}

#[test]
fn token_flags_jsdoc_tag_invalid_terminator() {
    let mut s = Scanner::new("/**\n * @deprecatedX\n */\nlet x");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::LetKeyword);
    assert!(!s.has_preceding_jsdoc_with_deprecated_tag());
}

#[test]
fn token_flags_jsdoc_tag_at_end_of_string() {
    let mut s = Scanner::new("/**@deprecated*/\nlet x");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::LetKeyword);
    assert!(s.has_preceding_jsdoc_with_deprecated_tag());
}

#[test]
fn token_flags_jsdoc_flags_reset_between_tokens() {
    let mut s = Scanner::new("/** @deprecated */\nlet x\nlet y");
    s.scan();
    assert!(s.has_preceding_jsdoc_with_deprecated_tag());
    s.scan();
    s.scan();
    assert!(!s.has_preceding_jsdoc_comment());
    assert!(!s.has_preceding_jsdoc_with_deprecated_tag());
}

#[test]
fn token_flags_jsdoc_leading_asterisk_consumed() {
    let mut s = Scanner::new("\n* x");
    s.set_skip_jsdoc_leading_asterisks(true);
    s.scan();

    assert_eq!(s.token(), SyntaxKind::Identifier);
    assert_eq!(s.token_text(), "x");
    assert!(s.has_preceding_jsdoc_leading_asterisks());
}

#[test]
fn token_flags_jsdoc_leading_asterisk_no_line_break() {
    let mut s = Scanner::new("* x");
    s.set_skip_jsdoc_leading_asterisks(true);
    s.scan();
    assert_eq!(s.token(), SyntaxKind::AsteriskToken);
    assert!(!s.has_preceding_jsdoc_leading_asterisks());
}

#[test]
fn token_flags_jsdoc_leading_asterisk_not_active() {
    let mut s = Scanner::new("\n* x");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::AsteriskToken);
    assert!(!s.has_preceding_jsdoc_leading_asterisks());
}

#[test]
fn token_flags_jsdoc_leading_asterisk_double_star_not_consumed() {
    let mut s = Scanner::new("\n** x");
    s.set_skip_jsdoc_leading_asterisks(true);
    s.scan();
    assert_eq!(s.token(), SyntaxKind::AsteriskAsteriskToken);
    assert!(!s.has_preceding_jsdoc_leading_asterisks());
}

#[test]
fn token_flags_jsdoc_leading_asterisk_star_equals_not_consumed() {
    let mut s = Scanner::new("\n*= x");
    s.set_skip_jsdoc_leading_asterisks(true);
    s.scan();
    assert_eq!(s.token(), SyntaxKind::AsteriskEqualsToken);
    assert!(!s.has_preceding_jsdoc_leading_asterisks());
}

#[test]
fn token_flags_jsdoc_leading_asterisk_only_first_consumed() {
    let mut s = Scanner::new("\n* * x");
    s.set_skip_jsdoc_leading_asterisks(true);
    s.scan();
    assert_eq!(s.token(), SyntaxKind::AsteriskToken);
    assert!(s.has_preceding_jsdoc_leading_asterisks());
}

#[test]
fn token_flags_jsdoc_leading_asterisk_counter_nesting() {
    let mut s = Scanner::new("\n* x");
    s.set_skip_jsdoc_leading_asterisks(true);
    s.set_skip_jsdoc_leading_asterisks(false);
    s.scan();
    assert_eq!(s.token(), SyntaxKind::AsteriskToken);
    assert!(!s.has_preceding_jsdoc_leading_asterisks());
}

#[test]
fn has_jsdoc_tag_helper() {
    assert!(has_jsdoc_tag("deprecated", &["deprecated"]));
    assert!(has_jsdoc_tag("deprecated foo", &["deprecated"]));
    assert!(has_jsdoc_tag("deprecated\tfoo", &["deprecated"]));
    assert!(has_jsdoc_tag("deprecated\nfoo", &["deprecated"]));
    assert!(has_jsdoc_tag("deprecated*foo", &["deprecated"]));
    assert!(has_jsdoc_tag("deprecated}foo", &["deprecated"]));
    assert!(has_jsdoc_tag("see", &["see", "link"]));
    assert!(has_jsdoc_tag("link foo", &["see", "link"]));
    assert!(has_jsdoc_tag(
        "linkcode foo",
        &["see", "link", "linkcode", "linkplain"]
    ));

    assert!(!has_jsdoc_tag("deprecatedX", &["deprecated"]));
    assert!(!has_jsdoc_tag("dep", &["deprecated"]));
    assert!(!has_jsdoc_tag("foo", &["deprecated"]));
}

#[test]
fn scan_jsdoc_comment_for_tags_helper() {
    assert_eq!(
        scan_jsdoc_comment_for_tags("/** @deprecated */"),
        TOKEN_FLAGS_PRECEDING_JSDOC_WITH_DEPRECATED
    );
    assert_eq!(
        scan_jsdoc_comment_for_tags("/** @see foo */"),
        TOKEN_FLAGS_PRECEDING_JSDOC_WITH_SEE_OR_LINK
    );
    assert_eq!(
        scan_jsdoc_comment_for_tags("/** @deprecated @see foo */"),
        TOKEN_FLAGS_PRECEDING_JSDOC_WITH_DEPRECATED | TOKEN_FLAGS_PRECEDING_JSDOC_WITH_SEE_OR_LINK
    );
    assert_eq!(
        scan_jsdoc_comment_for_tags("/** no tags */"),
        TOKEN_FLAGS_NONE
    );

    assert!(token_flags_contains(
        scan_jsdoc_comment_for_tags("/** {@link foo} */"),
        TOKEN_FLAGS_PRECEDING_JSDOC_WITH_SEE_OR_LINK
    ));
}

#[test]
fn token_flags_unicode_escape() {
    let mut s = Scanner::new("\"\\u00a0\"");
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_UNICODE_ESCAPE
    ));
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
    ));
}

#[test]
fn token_flags_extended_unicode_escape() {
    let mut s = Scanner::new("\"\\u{10ffff}\"");
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_EXTENDED_UNICODE_ESCAPE
    ));
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
    ));
}

#[test]
fn token_flags_hex_escape() {
    let mut s = Scanner::new("\"\\xa0\"");
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_HEX_ESCAPE
    ));
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
    ));
}

#[test]
fn token_flags_invalid_hex_escape() {
    let mut s = Scanner::new("\"\\xz\"");
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
    ));
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_HEX_ESCAPE
    ));
}

#[test]
fn token_flags_invalid_unicode_escape() {
    let mut s = Scanner::new("\"\\u00\"");
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
    ));
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_UNICODE_ESCAPE
    ));
}

#[test]
fn token_flags_invalid_extended_unicode_escape() {
    let mut s = Scanner::new("\"\\u{}\"");
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
    ));
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_EXTENDED_UNICODE_ESCAPE
    ));
}

#[test]
fn token_flags_octal_escape_invalid() {
    let mut s = Scanner::new("\"\\01\"");
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
    ));
}

#[test]
fn token_flags_escape_eight_nine_invalid() {
    let mut s = Scanner::new("\"\\8\"");
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
    ));
}

#[test]
fn token_flags_nul_escape_not_invalid() {
    let mut s = Scanner::new("\"\\0\"");
    s.scan();
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE
    ));
}

#[test]
fn token_flags_contains_separator_decimal() {
    let mut s = Scanner::new("1_000");
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_SEPARATOR
    ));
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR
    ));
}

#[test]
fn token_flags_contains_separator_hex() {
    let mut s = Scanner::new("0xFF_FF");
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_SEPARATOR
    ));
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR
    ));
}

#[test]
fn token_flags_contains_separator_binary() {
    let mut s = Scanner::new("0b1010_0101");
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_SEPARATOR
    ));
}

#[test]
fn token_flags_invalid_separator_consecutive() {
    let mut s = Scanner::new("1__000");
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_SEPARATOR
    ));
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR
    ));
}

#[test]
fn token_flags_invalid_separator_trailing() {
    let mut s = Scanner::new("1000_");
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_SEPARATOR
    ));
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR
    ));
}

#[test]
fn token_flags_no_separator_plain_number() {
    let mut s = Scanner::new("12345");
    s.scan();
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_SEPARATOR
    ));
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR
    ));
}

#[test]
fn token_flags_string_literal_flags_mask() {
    let mut s = Scanner::new("'\\x41\\u0041'");
    s.scan();
    let flags = s.token_flags();
    assert!(token_flags_contains(flags, TOKEN_FLAGS_HEX_ESCAPE));
    assert!(token_flags_contains(flags, TOKEN_FLAGS_UNICODE_ESCAPE));
    assert!(token_flags_contains(flags, TOKEN_FLAGS_SINGLE_QUOTE));

    assert!(token_flags_intersects(
        flags,
        TOKEN_FLAGS_STRING_LITERAL_FLAGS
    ));
}

#[test]
fn token_flags_numeric_literal_flags_mask() {
    let mut s = Scanner::new("0xFF_FF");
    s.scan();
    let flags = s.token_flags();
    assert!(token_flags_contains(flags, TOKEN_FLAGS_HEX_SPECIFIER));
    assert!(token_flags_contains(flags, TOKEN_FLAGS_CONTAINS_SEPARATOR));

    assert!(token_flags_intersects(
        flags,
        TOKEN_FLAGS_NUMERIC_LITERAL_FLAGS
    ));
}

#[test]
fn legacy_octal_literal_sets_octal_flag() {
    let mut s = Scanner::new("0777");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::NumericLiteral);
    assert!(token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
    let errors = s.take_errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, DiagnosticKind::OctalLiteralNotAllowed);
    assert_eq!(errors[0].pos, 0);
    assert_eq!(errors[0].length, 4);
}

#[test]
fn legacy_octal_literal_single_digit() {
    let mut s = Scanner::new("00");
    s.scan();
    assert!(token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
    let errors = s.take_errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, DiagnosticKind::OctalLiteralNotAllowed);
}

#[test]
fn leading_zero_non_octal_sets_leading_zero_flag() {
    let mut s = Scanner::new("0888");
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_LEADING_ZERO
    ));
    assert!(!token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
    let errors = s.take_errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, DiagnosticKind::DecimalWithLeadingZero);
    assert_eq!(errors[0].pos, 0);
    assert_eq!(errors[0].length, 4);
}

#[test]
fn plain_zero_no_flags() {
    let mut s = Scanner::new("0");
    s.scan();
    assert!(!token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_LEADING_ZERO
    ));
    assert!(s.take_errors().is_empty());
}

#[test]
fn zero_with_fraction_no_flags() {
    let mut s = Scanner::new("0.5");
    s.scan();
    assert!(!token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_LEADING_ZERO
    ));
    assert!(s.take_errors().is_empty());
}

#[test]
fn zero_with_exponent_no_flags() {
    let mut s = Scanner::new("0e5");
    s.scan();
    assert!(!token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
    assert!(!token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_LEADING_ZERO
    ));
    assert!(s.take_errors().is_empty());
}

#[test]
fn zero_bigint_no_flags() {
    let mut s = Scanner::new("0n");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::BigIntLiteral);
    assert!(!token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
    assert!(s.take_errors().is_empty());
}

#[test]
fn zero_separator_after_leading_zero() {
    let mut s = Scanner::new("0_123");
    s.scan();
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_SEPARATOR
    ));
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_CONTAINS_INVALID_SEPARATOR
    ));
    let errors = s.take_errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, DiagnosticKind::NumericSeparatorNotAllowed);
    assert_eq!(errors[0].pos, 1);
}

#[test]
fn legacy_octal_with_minus_prefix() {
    let mut s = Scanner::new("-0777");
    s.scan();
    assert_eq!(s.token(), SyntaxKind::MinusToken);
    s.scan();
    assert!(token_flags_contains(s.token_flags(), TOKEN_FLAGS_OCTAL));
    let errors = s.take_errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, DiagnosticKind::OctalLiteralNotAllowed);

    assert_eq!(errors[0].pos, 0);
    assert_eq!(errors[0].length, 5);
}

#[test]
fn jsdoc_token_at_sign() {
    let mut s = Scanner::new("@param");
    assert_eq!(s.scan_jsdoc_token(), SyntaxKind::AtToken);
}

#[test]
fn jsdoc_token_asterisk() {
    let mut s = Scanner::new("*");
    assert_eq!(s.scan_jsdoc_token(), SyntaxKind::AsteriskToken);
}

#[test]
fn jsdoc_token_identifier_and_keyword() {
    let mut s = Scanner::new("param");
    assert_eq!(s.scan_jsdoc_token(), SyntaxKind::Identifier);
    let mut s = Scanner::new("return");
    assert_eq!(s.scan_jsdoc_token(), SyntaxKind::ReturnKeyword);
}

#[test]
fn jsdoc_token_identifier_with_dash() {
    let mut s = Scanner::new("custom-tag");
    assert_eq!(s.scan_jsdoc_token(), SyntaxKind::Identifier);
    assert_eq!(s.token_text(), "custom-tag");
}

#[test]
fn jsdoc_token_whitespace() {
    let mut s = Scanner::new("   \t  ");
    assert_eq!(s.scan_jsdoc_token(), SyntaxKind::WhitespaceTrivia);
}

#[test]
fn jsdoc_token_newline() {
    let mut s = Scanner::new("\n");
    assert_eq!(s.scan_jsdoc_token(), SyntaxKind::NewLineTrivia);
    assert!(token_flags_contains(
        s.token_flags(),
        TOKEN_FLAGS_PRECEDING_LINE_BREAK
    ));
}

#[test]
fn jsdoc_token_crlf_newline() {
    let mut s = Scanner::new("\r\n");
    assert_eq!(s.scan_jsdoc_token(), SyntaxKind::NewLineTrivia);
}

#[test]
fn jsdoc_token_braces_and_brackets() {
    let mut s = Scanner::new("{");
    assert_eq!(s.scan_jsdoc_token(), SyntaxKind::OpenBraceToken);
    let mut s = Scanner::new("}");
    assert_eq!(s.scan_jsdoc_token(), SyntaxKind::CloseBraceToken);
    let mut s = Scanner::new("[");
    assert_eq!(s.scan_jsdoc_token(), SyntaxKind::OpenBracketToken);
    let mut s = Scanner::new("]");
    assert_eq!(s.scan_jsdoc_token(), SyntaxKind::CloseBracketToken);
}

#[test]
fn jsdoc_token_punctuation() {
    let mut s = Scanner::new("(");
    assert_eq!(s.scan_jsdoc_token(), SyntaxKind::OpenParenToken);
    let mut s = Scanner::new("`");
    assert_eq!(s.scan_jsdoc_token(), SyntaxKind::BacktickToken);
    let mut s = Scanner::new("#");
    assert_eq!(s.scan_jsdoc_token(), SyntaxKind::HashToken);
}

#[test]
fn jsdoc_token_eof() {
    let mut s = Scanner::new("");
    assert_eq!(s.scan_jsdoc_token(), SyntaxKind::EndOfFile);
}

#[test]
fn jsdoc_can_follow_at_identifier() {
    let s = Scanner::new("param");
    assert!(s.can_follow_jsdoc_at());
}

#[test]
fn jsdoc_can_follow_at_whitespace() {
    let s = Scanner::new(" ");
    assert!(s.can_follow_jsdoc_at());
}

#[test]
fn jsdoc_can_follow_at_eof() {
    let s = Scanner::new("");
    assert!(s.can_follow_jsdoc_at());
}

#[test]
fn jsdoc_can_follow_at_digit_false() {
    let s = Scanner::new("1abc");
    assert!(!s.can_follow_jsdoc_at());
}

#[test]
fn jsdoc_comment_text_token_prose() {
    let mut s = Scanner::new("This is a description. ");
    assert_eq!(
        s.scan_jsdoc_comment_text_token(false),
        SyntaxKind::JSDocCommentTextToken
    );
    assert_eq!(s.token_text(), "This is a description. ");
}

#[test]
fn jsdoc_comment_text_token_stops_at_brace() {
    let mut s = Scanner::new("before {type} after");
    assert_eq!(
        s.scan_jsdoc_comment_text_token(false),
        SyntaxKind::JSDocCommentTextToken
    );
    assert_eq!(s.token_text(), "before ");

    assert_eq!(s.scan_jsdoc_token(), SyntaxKind::OpenBraceToken);
}

#[test]
fn jsdoc_comment_text_token_stops_at_newline() {
    let mut s = Scanner::new("line1\nline2");
    assert_eq!(
        s.scan_jsdoc_comment_text_token(false),
        SyntaxKind::JSDocCommentTextToken
    );
    assert_eq!(s.token_text(), "line1");
}

#[test]
fn jsdoc_comment_text_token_at_tag_boundary() {
    let mut s = Scanner::new("text @param");
    assert_eq!(
        s.scan_jsdoc_comment_text_token(false),
        SyntaxKind::JSDocCommentTextToken
    );
    assert_eq!(s.token_text(), "text ");
}

#[test]
fn jsdoc_comment_text_token_in_backticks_ignores_at_and_brace() {
    let mut s = Scanner::new("code {@code x} more");
    assert_eq!(
        s.scan_jsdoc_comment_text_token(true),
        SyntaxKind::JSDocCommentTextToken
    );

    assert_eq!(s.token_text(), "code {@code x} more");
}

#[test]
fn jsdoc_comment_text_token_empty_falls_through() {
    let mut s = Scanner::new("{");
    assert_eq!(
        s.scan_jsdoc_comment_text_token(false),
        SyntaxKind::OpenBraceToken
    );
}

#[test]
fn scan_string_preserves_lone_surrogates() {
    let input = r#""🦀\ud7ff\ud800\ud801\uD83E\uDD80""#;
    let mut s = Scanner::new(input);
    assert_eq!(s.scan(), SyntaxKind::StringLiteral);

    let value = s.token_value();
    assert!(value.contains('🦀'));
    assert!(value.contains('\u{D7FF}'));

    let fffd_count = value.chars().filter(|&c| c == '\u{FFFD}').count();
    assert_eq!(fffd_count, 4);
}
