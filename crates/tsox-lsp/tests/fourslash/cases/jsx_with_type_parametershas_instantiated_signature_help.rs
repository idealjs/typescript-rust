use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn jsx_with_type_parametershas_instantiated_signature_help() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"declare namespace JSX {
    interface Element {
        render(): Element | string | false;
    }
}

function SFC<T>(_props: Record<string, T>) {
    return '';
}

(</*1*/SFC/>);
(</*2*/SFC<string>/>);"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "SFC(_props: Record<string, unkn
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{Text: "SFC(_props: Record<string, stri
}
