use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoExists"]
#[test]
fn quick_info_with_nested_destructured_parameter_in_lambda() {
    let content = r#"// @filename: a.tsx
import * as React from 'react';
interface SomeInterface {
    someBoolean: boolean,
    someString: string;
}
interface SomeProps {
    someProp: SomeInterface;
}
export const /*1*/SomeStatelessComponent = ({someProp: { someBoolean, someString}}: SomeProps) => (<div>{` + "`" + `${someBoolean}${someString}` + "`" + `});"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyQuickInfoExists"); // f.VerifyQuickInfoExists(t)
}
