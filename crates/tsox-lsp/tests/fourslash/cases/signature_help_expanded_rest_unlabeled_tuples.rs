use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifySignatureHelp"]
#[test]
fn signature_help_expanded_rest_unlabeled_tuples() {
    let content = r#"export function complex(item: string, another: string, ...rest: [] | [object, (err: Error) => void] | [(err: Error) => void, ...object[]]) {
    
}

complex(/*1*/);
complex("ok", "ok", /*2*/);
complex("ok", "ok", e => void e, {}, /*3*/);"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "complex(item: string, another: 
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "complex(item: string, another: 
    fourslash::go_to_marker(&mut s, "3");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "complex(item: string, another: 
}
