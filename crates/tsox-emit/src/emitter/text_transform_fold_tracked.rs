pub(crate) fn fold_expression_newlines_tracked(
    text: &str,
    src_offsets: &[u32],
) -> (String, Vec<u32>) {
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
