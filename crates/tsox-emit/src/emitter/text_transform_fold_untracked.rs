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
