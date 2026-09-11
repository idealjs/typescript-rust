use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsx_element_missing_opening_tag_no_crash() {
    let content = r#"//@Filename: file.tsx
declare function Foo(): any;
let x = <></Fo/*$*/o>;"#;
    let mut s = Session::new_for_test("jsxElementMissingOpeningTagNoCrash", content);
    fourslash::verify_quick_info_at(&mut s, "$", "let Foo: any", "");
}
