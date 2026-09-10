use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_jsx_not_set() {
    let content = r#"// @allowJs: true
// @Filename: /foo.jsx
const /*def*/Foo = () => (
    <div>foo</div>
);
export default Foo;
// @Filename: /bar.jsx
import Foo from './foo';
const a = <[|/*use*/Foo|] />"#;
    let mut s = Session::new_for_test("goToDefinitionJsxNotSet", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "use")
}
