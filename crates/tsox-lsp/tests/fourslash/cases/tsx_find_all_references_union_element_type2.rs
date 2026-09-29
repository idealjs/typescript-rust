use tsox_lsp::fourslash::Session;


#[test]
fn tsx_find_all_references_union_element_type2() {
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
/*1*/var /*2*/RCComp = RC1 || RC2;
/*3*/</*4*/RCComp />"#;
    let _s = Session::new_for_test("tsxFindAllReferencesUnionElementType2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
