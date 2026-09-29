use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_expanded_rest_tuples() {
    let content = r#"export function complex(item: string, another: string, ...rest: [] | [settings: object, errorHandler: (err: Error) => void] | [errorHandler: (err: Error) => void, ...mixins: object[]]) {
    
}

complex(/*1*/);
complex("ok", "ok", /*2*/);
complex("ok", "ok", e => void e, {}, /*3*/);"#;
    let mut s = Session::new_for_test("signatureHelpExpandedRestTuples", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "complex(item: string, another: 
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "complex(item: string, another: 
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "complex(item: string, another: 
}
