use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_rest_args1() {
    let content = r#"function fn(a: number, b: number, c: number) {}
const a = [1, 2] as const;
const b = [1] as const;

fn(...a, /*1*/);
fn(/*2*/, ...a);

fn(...b, /*3*/);
fn(/*4*/, ...b, /*5*/);"#;
    let mut s = Session::new_for_test("signatureHelpRestArgs1", content);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
