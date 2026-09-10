use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_jsx_call() {
    let content = r#"// @filename: ./test.tsx
interface FC<P = {}> {
    (props: P, context?: any): string;
}

const Thing: FC = (props) => <div></div>;
const HelloWorld = () => <[|/**/Thing|] />;"#;
    let mut s = Session::new_for_test("goToDefinitionJsxCall", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "")
}
