
use super::*;
use super::commonjs::*;

pub(crate) fn add_implicit_semicolons(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for line in text.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            result.push('\n');
            continue;
        }
        let last = trimmed.chars().last().unwrap_or(' ');

        let skip = matches!(
            last,
            '{' | '(' | '[' | ',' | ';' | ':' | '.' | '|' | '&' | '=' | '>' | '?'
        ) || trimmed.ends_with("=>")
            || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.ends_with("*/");
        if skip {
            result.push_str(trimmed);
        } else if last == '}' {

            result.push_str(trimmed);
        } else {
            result.push_str(trimmed);
            result.push(';');
        }
        result.push('\n');
    }
    if !text.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }
    result
}

#[allow(dead_code)]
pub(crate) fn normalize_js_output(text: &str) -> String {
    let folded = fold_expression_newlines(text);
    let reindented = reindent_and_dedup(&folded);
    add_implicit_semicolons(&reindented)
}

pub(crate) fn rewrite_import_extensions_tracked(text: &str, src_offsets: &[u32]) -> (String, Vec<u32>) {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out_text = String::with_capacity(text.len());
    let mut out_offsets = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        if i + 5 <= n
            && chars[i] == 'f'
            && chars[i + 1] == 'r'
            && chars[i + 2] == 'o'
            && chars[i + 3] == 'm'
            && chars[i + 4] == ' '
        {
            i = copy_string_literal_tracked(
                &chars,
                src_offsets,
                i,
                &mut out_text,
                &mut out_offsets,
            );
        } else if i + 7 <= n
            && chars[i] == 'i'
            && chars[i + 1] == 'm'
            && chars[i + 2] == 'p'
            && chars[i + 3] == 'o'
            && chars[i + 4] == 'r'
            && chars[i + 5] == 't'
            && chars[i + 6] == '('
        {
            for j in 0..7 {
                out_text.push(chars[i + j]);
                out_offsets.push(src_offsets[i + j]);
            }
            i += 7;
            while i < n && chars[i].is_ascii_whitespace() {
                out_text.push(chars[i]);
                out_offsets.push(src_offsets[i]);
                i += 1;
            }
            if i < n && (chars[i] == '"' || chars[i] == '\'') {
                i = copy_string_literal_tracked(
                    &chars,
                    src_offsets,
                    i,
                    &mut out_text,
                    &mut out_offsets,
                );
            }
        } else {
            out_text.push(chars[i]);
            out_offsets.push(src_offsets[i]);
            i += 1;
        }
    }
    (out_text, out_offsets)
}

pub(crate) fn copy_string_literal_tracked(
    chars: &[char],
    src_offsets: &[u32],
    start: usize,
    out_text: &mut String,
    out_offsets: &mut Vec<u32>,
) -> usize {

    let mut i = start;
    if chars[i] == 'f' {
        for _ in 0..5 {
            out_text.push(chars[i]);
            out_offsets.push(src_offsets[i]);
            i += 1;
        }
        while i < chars.len() && chars[i].is_ascii_whitespace() {
            out_text.push(chars[i]);
            out_offsets.push(src_offsets[i]);
            i += 1;
        }
    }

    if i < chars.len() && (chars[i] == '"' || chars[i] == '\'') {
        let quote = chars[i];
        out_text.push(quote);
        out_offsets.push(src_offsets[i]);
        i += 1;
        let spec_start = i;
        while i < chars.len() && chars[i] != quote {
            i += 1;
        }
        let specifier: String = chars[spec_start..i].iter().collect();
        let rewritten = rewrite_one_specifier(&specifier);
        let spec_len = i - spec_start;
        for (j, rc) in rewritten.chars().enumerate() {
            out_text.push(rc);
            if j < spec_len {
                out_offsets.push(src_offsets[spec_start + j]);
            } else {
                out_offsets.push(src_offsets[spec_start + spec_len - 1]);
            }
        }
        if i < chars.len() {
            out_text.push(chars[i]);
            out_offsets.push(src_offsets[i]);
            i += 1;
        }
    }
    i
}

pub(crate) fn fold_expression_newlines_tracked(text: &str, src_offsets: &[u32]) -> (String, Vec<u32>) {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out: Vec<char> = Vec::with_capacity(n);
    let mut out_idx: Vec<usize> = Vec::with_capacity(n);

    #[derive(Clone, Copy, PartialEq)]
    enum SCtx {
        Single,
        Double,
        Template,
        LineComment,
        BlockComment,
    }
    #[derive(Clone, Copy)]
    enum Group {
        Paren(bool),
        Bracket(bool),
        Brace,
        TmplInterp,
    }

    let mut sctx: Vec<SCtx> = Vec::new();
    let mut groups: Vec<Group> = Vec::new();

    let mut i = 0;
    while i < n {
        let c = chars[i];

        if let Some(&ctx) = sctx.last() {
            match ctx {
                SCtx::Single => {
                    out.push(c);
                    out_idx.push(i);
                    if c == '\\' {
                        i += 1;
                        if i < n {
                            out.push(chars[i]);
                            out_idx.push(i);
                            i += 1;
                        }
                        continue;
                    }
                    if c == '\'' {
                        sctx.pop();
                    }
                    i += 1;
                    continue;
                }
                SCtx::Double => {
                    out.push(c);
                    out_idx.push(i);
                    if c == '\\' {
                        i += 1;
                        if i < n {
                            out.push(chars[i]);
                            out_idx.push(i);
                            i += 1;
                        }
                        continue;
                    }
                    if c == '"' {
                        sctx.pop();
                    }
                    i += 1;
                    continue;
                }
                SCtx::Template => {
                    out.push(c);
                    out_idx.push(i);
                    if c == '\\' {
                        i += 1;
                        if i < n {
                            out.push(chars[i]);
                            out_idx.push(i);
                            i += 1;
                        }
                        continue;
                    }
                    if c == '`' {
                        sctx.pop();
                        i += 1;
                        continue;
                    }
                    if c == '$' && i + 1 < n && chars[i + 1] == '{' {
                        out.push('{');
                        out_idx.push(i + 1);
                        sctx.pop();
                        groups.push(Group::TmplInterp);
                        i += 2;
                        continue;
                    }
                    i += 1;
                    continue;
                }
                SCtx::LineComment => {
                    if c == '\n' || c == '\r' {
                        sctx.pop();

                    } else {
                        out.push(c);
                        out_idx.push(i);
                        i += 1;
                        continue;
                    }
                }
                SCtx::BlockComment => {
                    out.push(c);
                    out_idx.push(i);
                    if c == '*' && i + 1 < n && chars[i + 1] == '/' {
                        out.push('/');
                        out_idx.push(i + 1);
                        sctx.pop();
                        i += 2;
                        continue;
                    }
                    i += 1;
                    continue;
                }
            }
        }

        if c == '\'' {
            sctx.push(SCtx::Single);
            out.push(c);
            out_idx.push(i);
            i += 1;
            continue;
        }
        if c == '"' {
            sctx.push(SCtx::Double);
            out.push(c);
            out_idx.push(i);
            i += 1;
            continue;
        }
        if c == '`' {
            sctx.push(SCtx::Template);
            out.push(c);
            out_idx.push(i);
            i += 1;
            continue;
        }
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            sctx.push(SCtx::LineComment);
            out.push('/');
            out_idx.push(i);
            out.push('/');
            out_idx.push(i + 1);
            i += 2;
            continue;
        }
        if c == '/' && i + 1 < n && chars[i + 1] == '*' {
            sctx.push(SCtx::BlockComment);
            out.push('/');
            out_idx.push(i);
            out.push('*');
            out_idx.push(i + 1);
            i += 2;
            continue;
        }

        if c == '(' {
            groups.push(Group::Paren(false));
            out.push(c);
            out_idx.push(i);
            i += 1;
            continue;
        }
        if c == '[' {
            groups.push(Group::Bracket(false));
            out.push(c);
            out_idx.push(i);
            i += 1;
            continue;
        }
        if c == '{' {
            groups.push(Group::Brace);
            out.push(c);
            out_idx.push(i);
            i += 1;
            continue;
        }
        if c == ')' {
            if let Some(Group::Paren(folded)) = groups.last().copied() {
                groups.pop();
                if folded {
                    drop_trailing_idx(&mut out, &mut out_idx);
                }
            }
            out.push(c);
            out_idx.push(i);
            i += 1;
            continue;
        }
        if c == ']' {
            if let Some(Group::Bracket(folded)) = groups.last().copied() {
                groups.pop();
                if folded {
                    drop_trailing_idx(&mut out, &mut out_idx);
                }
            }
            out.push(c);
            out_idx.push(i);
            i += 1;
            continue;
        }
        if c == '}' {
            match groups.last() {
                Some(Group::TmplInterp) => {
                    groups.pop();
                    sctx.push(SCtx::Template);
                    out.push(c);
                    out_idx.push(i);
                    i += 1;
                    continue;
                }
                Some(Group::Brace) => {
                    groups.pop();
                }
                _ => {}
            }
            out.push(c);
            out_idx.push(i);
            i += 1;
            continue;
        }

        if c == '\n' || c == '\r' {
            let do_fold = matches!(
                groups.last(),
                Some(Group::Paren(_)) | Some(Group::Bracket(_))
            );
            if do_fold {
                if let Some(g) = groups.last_mut() {
                    if let Group::Paren(f) | Group::Bracket(f) = g {
                        *f = true;
                    }
                }
                while let Some(&ch) = out.last() {
                    if ch == ' ' || ch == '\t' {
                        out.pop();
                        out_idx.pop();
                    } else {
                        break;
                    }
                }
                i += 1;
                if i < n && chars[i - 1] == '\r' && chars[i] == '\n' {
                    i += 1;
                }
                while i < n && (chars[i] == ' ' || chars[i] == '\t') {
                    i += 1;
                }
            } else {
                out.push('\n');
                out_idx.push(i);
                i += 1;
                if i < n && chars[i - 1] == '\r' && chars[i] == '\n' {
                    i += 1;
                }
            }
            continue;
        }

        out.push(c);
        out_idx.push(i);
        i += 1;
    }

    let result_text: String = out.into_iter().collect();
    let result_offsets: Vec<u32> = out_idx.iter().map(|&idx| src_offsets[idx]).collect();
    (result_text, result_offsets)
}

pub(crate) fn drop_trailing_idx(out: &mut Vec<char>, out_idx: &mut Vec<usize>) {
    while let Some(&ch) = out.last() {
        if ch == ' ' || ch == '\t' {
            out.pop();
            out_idx.pop();
        } else {
            break;
        }
    }
    if out.last() == Some(&',') {
        out.pop();
        out_idx.pop();
    }
}

pub(crate) fn reindent_and_dedup_tracked(folded: &str, src_offsets: &[u32]) -> (String, Vec<u32>) {
    let chars: Vec<char> = folded.chars().collect();
    let n = chars.len();
    let mut out_text = String::with_capacity(folded.len());
    let mut out_offsets: Vec<u32> = Vec::new();
    let mut depth: i32 = 0;
    let had_trailing_newline = n > 0 && chars[n - 1] == '\n';

    let mut i = 0;
    while i < n {
        let line_start = i;
        while i < n && chars[i] != '\n' {
            i += 1;
        }
        let line_end = i;
        let newline_idx = if i < n && chars[i] == '\n' {
            Some(i)
        } else {
            None
        };
        if i < n && chars[i] == '\n' {
            i += 1;
        }

        let mut content_start = line_start;
        while content_start < line_end && chars[content_start].is_whitespace() {
            content_start += 1;
        }
        let mut content_end = line_end;
        while content_end > content_start && chars[content_end - 1].is_whitespace() {
            content_end -= 1;
        }

        if content_start >= content_end {
            continue;
        }

        let starts_with_close = chars[content_start] == '}';
        let indent_depth = (depth - if starts_with_close { 1 } else { 0 }).max(0);
        for _ in 0..indent_depth {
            out_text.push_str("    ");
            for _ in 0..4 {
                out_offsets.push(UNMAPPED);
            }
        }
        for j in content_start..content_end {
            out_text.push(chars[j]);
            out_offsets.push(src_offsets[j]);
        }
        out_text.push('\n');
        if let Some(nl) = newline_idx {
            out_offsets.push(src_offsets[nl]);
        } else {
            out_offsets.push(src_offsets[content_end - 1]);
        }

        let content: String = chars[content_start..content_end].iter().collect();
        depth += brace_delta(&content);
        if depth < 0 {
            depth = 0;
        }
    }

    if !had_trailing_newline && out_text.ends_with('\n') {
        out_text.pop();
        out_offsets.pop();
    }
    (out_text, out_offsets)
}

pub(crate) fn add_implicit_semicolons_tracked(text: &str, src_offsets: &[u32]) -> (String, Vec<u32>) {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out_text = String::with_capacity(text.len());
    let mut out_offsets: Vec<u32> = Vec::new();
    let had_trailing_newline = n > 0 && chars[n - 1] == '\n';

    let mut i = 0;
    while i < n {
        let line_start = i;
        while i < n && chars[i] != '\n' {
            i += 1;
        }
        let line_end = i;
        let has_newline = i < n && chars[i] == '\n';
        if has_newline {
            i += 1;
        }

        let mut content_end = line_end;
        while content_end > line_start && chars[content_end - 1].is_whitespace() {
            content_end -= 1;
        }

        if content_end == line_start {
            out_text.push('\n');
            if has_newline {
                out_offsets.push(src_offsets[line_end]);
            } else {
                out_offsets.push(UNMAPPED);
            }
            continue;
        }

        let last = chars[content_end - 1];
        let trimmed_str: String = chars[line_start..content_end].iter().collect();
        let skip = matches!(
            last,
            '{' | '(' | '[' | ',' | ';' | ':' | '.' | '|' | '&' | '=' | '>' | '?'
        ) || trimmed_str.ends_with("=>")
            || trimmed_str.starts_with("//")
            || trimmed_str.starts_with("/*")
            || trimmed_str.ends_with("*/");

        for j in line_start..content_end {
            out_text.push(chars[j]);
            out_offsets.push(src_offsets[j]);
        }

        if !skip && last != '}' {
            out_text.push(';');
            out_offsets.push(UNMAPPED);
        }

        out_text.push('\n');
        if has_newline {
            out_offsets.push(src_offsets[line_end]);
        } else {
            out_offsets.push(UNMAPPED);
        }
    }

    if !had_trailing_newline && out_text.ends_with('\n') {
        out_text.pop();
        out_offsets.pop();
    }
    (out_text, out_offsets)
}

pub(crate) fn normalize_js_output_tracked(text: &str, src_offsets: &[u32]) -> (String, Vec<u32>) {
    let (text, offsets) = fold_expression_newlines_tracked(text, src_offsets);
    let (text, offsets) = reindent_and_dedup_tracked(&text, &offsets);
    add_implicit_semicolons_tracked(&text, &offsets)
}

#[allow(dead_code)]
pub(crate) fn fold_expression_newlines(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out: Vec<char> = Vec::with_capacity(n);

    #[derive(Clone, Copy, PartialEq)]
    enum SCtx {
        Single,
        Double,
        Template,
        LineComment,
        BlockComment,
    }

    #[derive(Clone, Copy)]
    enum Group {
        Paren(bool),
        Bracket(bool),
        Brace,
        TmplInterp,
    }

    let mut sctx: Vec<SCtx> = Vec::new();
    let mut groups: Vec<Group> = Vec::new();

    let mut i = 0;
    while i < n {
        let c = chars[i];

        if let Some(&ctx) = sctx.last() {
            match ctx {
                SCtx::Single => {
                    out.push(c);
                    if c == '\\' {
                        i += 1;
                        if i < n {
                            out.push(chars[i]);
                            i += 1;
                        }
                        continue;
                    }
                    if c == '\'' {
                        sctx.pop();
                    }
                    i += 1;
                    continue;
                }
                SCtx::Double => {
                    out.push(c);
                    if c == '\\' {
                        i += 1;
                        if i < n {
                            out.push(chars[i]);
                            i += 1;
                        }
                        continue;
                    }
                    if c == '"' {
                        sctx.pop();
                    }
                    i += 1;
                    continue;
                }
                SCtx::Template => {
                    out.push(c);
                    if c == '\\' {
                        i += 1;
                        if i < n {
                            out.push(chars[i]);
                            i += 1;
                        }
                        continue;
                    }
                    if c == '`' {
                        sctx.pop();
                        i += 1;
                        continue;
                    }
                    if c == '$' && i + 1 < n && chars[i + 1] == '{' {
                        out.push('{');
                        sctx.pop();
                        groups.push(Group::TmplInterp);
                        i += 2;
                        continue;
                    }
                    i += 1;
                    continue;
                }
                SCtx::LineComment => {
                    if c == '\n' || c == '\r' {
                        sctx.pop();

                    } else {
                        out.push(c);
                        i += 1;
                        continue;
                    }
                }
                SCtx::BlockComment => {
                    out.push(c);
                    if c == '*' && i + 1 < n && chars[i + 1] == '/' {
                        out.push('/');
                        sctx.pop();
                        i += 2;
                        continue;
                    }
                    i += 1;
                    continue;
                }
            }
        }

        if c == '\'' {
            sctx.push(SCtx::Single);
            out.push(c);
            i += 1;
            continue;
        }
        if c == '"' {
            sctx.push(SCtx::Double);
            out.push(c);
            i += 1;
            continue;
        }
        if c == '`' {
            sctx.push(SCtx::Template);
            out.push(c);
            i += 1;
            continue;
        }
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            sctx.push(SCtx::LineComment);
            out.push('/');
            out.push('/');
            i += 2;
            continue;
        }
        if c == '/' && i + 1 < n && chars[i + 1] == '*' {
            sctx.push(SCtx::BlockComment);
            out.push('/');
            out.push('*');
            i += 2;
            continue;
        }

        if c == '(' {
            groups.push(Group::Paren(false));
            out.push(c);
            i += 1;
            continue;
        }
        if c == '[' {
            groups.push(Group::Bracket(false));
            out.push(c);
            i += 1;
            continue;
        }
        if c == '{' {
            groups.push(Group::Brace);
            out.push(c);
            i += 1;
            continue;
        }
        if c == ')' {
            if let Some(Group::Paren(folded)) = groups.last().copied() {
                groups.pop();
                if folded {
                    drop_trailing_comma(&mut out);
                }
            }
            out.push(c);
            i += 1;
            continue;
        }
        if c == ']' {
            if let Some(Group::Bracket(folded)) = groups.last().copied() {
                groups.pop();
                if folded {
                    drop_trailing_comma(&mut out);
                }
            }
            out.push(c);
            i += 1;
            continue;
        }
        if c == '}' {
            match groups.last() {
                Some(Group::TmplInterp) => {
                    groups.pop();
                    sctx.push(SCtx::Template);
                    out.push(c);
                    i += 1;
                    continue;
                }
                Some(Group::Brace) => {
                    groups.pop();
                }
                _ => {}
            }
            out.push(c);
            i += 1;
            continue;
        }

        if c == '\n' || c == '\r' {
            let do_fold = matches!(
                groups.last(),
                Some(Group::Paren(_)) | Some(Group::Bracket(_))
            );
            if do_fold {
                if let Some(g) = groups.last_mut() {
                    if let Group::Paren(f) | Group::Bracket(f) = g {
                        *f = true;
                    }
                }

                while let Some(&ch) = out.last() {
                    if ch == ' ' || ch == '\t' {
                        out.pop();
                    } else {
                        break;
                    }
                }

                i += 1;
                if i < n && chars[i - 1] == '\r' && chars[i] == '\n' {
                    i += 1;
                }

                while i < n && (chars[i] == ' ' || chars[i] == '\t') {
                    i += 1;
                }
            } else {
                out.push('\n');
                i += 1;
                if i < n && chars[i - 1] == '\r' && chars[i] == '\n' {
                    i += 1;
                }
            }
            continue;
        }

        out.push(c);
        i += 1;
    }

    out.into_iter().collect()
}

#[allow(dead_code)]
pub(crate) fn drop_trailing_comma(out: &mut Vec<char>) {
    while let Some(&ch) = out.last() {
        if ch == ' ' || ch == '\t' {
            out.pop();
        } else {
            break;
        }
    }
    if out.last() == Some(&',') {
        out.pop();
    }
}

pub(crate) fn brace_delta(line: &str) -> i32 {
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    let mut delta = 0i32;
    let mut i = 0;
    while i < n {
        let c = chars[i];
        match c {
            '\'' | '"' | '`' => {
                let quote = c;
                i += 1;
                while i < n {
                    if chars[i] == '\\' {
                        i += 2;
                        continue;
                    }
                    if chars[i] == quote {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
            }
            '/' if i + 1 < n && chars[i + 1] == '/' => {

                break;
            }
            '/' if i + 1 < n && chars[i + 1] == '*' => {
                i += 2;
                while i < n {
                    if chars[i] == '*' && i + 1 < n && chars[i + 1] == '/' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
            }
            '{' => {
                delta += 1;
                i += 1;
            }
            '}' => {
                delta -= 1;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    delta
}

pub(crate) fn reindent_and_dedup(folded: &str) -> String {
    let mut out = String::with_capacity(folded.len());
    let mut depth: i32 = 0;
    let had_trailing_newline = folded.ends_with('\n');

    for raw_line in folded.split('\n') {
        let stripped = raw_line.trim();
        if stripped.is_empty() {
            continue;
        }
        let starts_with_close = stripped.starts_with('}');
        let indent_depth = (depth - if starts_with_close { 1 } else { 0 }).max(0);
        for _ in 0..indent_depth {
            out.push_str("    ");
        }
        out.push_str(stripped);
        out.push('\n');
        depth += brace_delta(stripped);
        if depth < 0 {
            depth = 0;
        }
    }

    if !had_trailing_newline && out.ends_with('\n') {
        out.pop();
    }
    out
}
