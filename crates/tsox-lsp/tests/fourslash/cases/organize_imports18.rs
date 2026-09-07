use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyOrganizeImports"]
#[test]
fn organize_imports18() {
    let content = r#"// @filename: /A.ts
export interface A {}
export function bFuncA(a: A) {}
// @filename: /B.ts
export interface B {}
export function bFuncB(b: B) {}
// @filename: /C.ts
export interface C {}
export function bFuncC(c: C) {}
// @filename: /test.ts
export { C } from "./C";
export { B } from "./B";
export { A } from "./A";

export { bFuncC } from "./C";
export { bFuncB } from "./B";
export { bFuncA } from "./A";"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/test.ts");
    fourslash::unsupported("VerifyOrganizeImports"); // f.VerifyOrganizeImports(t,
}
