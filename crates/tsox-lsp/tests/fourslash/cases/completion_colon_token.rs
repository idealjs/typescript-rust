use tsox_lsp::fourslash::Session;


#[test]
fn completion_colon_token() {
    let content = r#"
// @filename: /a.ts
:/*a*/

// @filename: /b.ts
function b(class: /*b*/) {}

// @filename: /c.ts
function c(enum: /*c*/) {}
"#;
    let _s = Session::new_for_test("completionColonToken", content);
    // TODO: for _, marker := range f.Ranges() {
}
