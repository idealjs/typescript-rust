use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn tsx_go_to_definition_intrinsics() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
        /*dt*/div: {
            /*pt*/name?: string;
            isOpen?: boolean;
        };
        /*st*/span: { n: string; };
    }
}
var x = <[|di/*ds*/v|] />;
var y = <[|s/*ss*/pan|] />;
var z = <div [|na/*ps*/me|]='hello' />;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "ds", "ss", "ps")
}
