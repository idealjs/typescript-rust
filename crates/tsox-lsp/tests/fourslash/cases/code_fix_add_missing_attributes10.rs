use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_add_missing_attributes10() {
    let content = r#"// @jsx: preserve
// @filename: foo.tsx
type A = 'a' | 'b' | 'c' | 'd' | 'e';
type B = 1 | 2 | 3;
type C = '@' | '!';
type D = ` + "`" + `${A}${Uppercase<A>}${B}${C}` + "`" + `;
const A = (props: { [K in D]: K }) =>
   <div {...props}></div>;

const Bar = () =>
   [|<A></A>|]"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t, "fixMissingAttributes")
}
