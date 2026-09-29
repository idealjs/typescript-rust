use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsx_attribute_completion_style_default() {
    let content = r#"// @Filename: foo.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
        foo: {
            prop_a: boolean;
            prop_b: string;
            prop_c: any;
            prop_d: { p1: string; }
            prop_e: string | undefined;
            prop_f: boolean | undefined | { p1: string; };
            prop_g: { p1: string; } | undefined;
            prop_h?: string;
            prop_i?: boolean;
            prop_j?: { p1: string; };
        }
    }
}

<foo [|prop_/**/|] />"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
