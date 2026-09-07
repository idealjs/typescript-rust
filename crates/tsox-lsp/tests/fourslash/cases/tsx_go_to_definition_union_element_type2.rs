use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn tsx_go_to_definition_union_element_type2() {
    let content = r#"//@Filename: file.tsx
// @jsx: preserve
// @noLib: true
class RC1 extends React.Component<{}, {}> {
    render() {
        return null;
    }
}
class RC2 extends React.Component<{}, {}> {
    render() {
        return null;
    }
    private method() { }
}
var /*pt1*/RCComp = RC1 || RC2;
<[|RC/*one*/Comp|] />"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "one")
}
