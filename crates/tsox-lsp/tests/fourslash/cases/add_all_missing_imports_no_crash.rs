use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixAll"]
#[test]
fn add_all_missing_imports_no_crash() {
    let content = r#"// @Filename: file1.ts
export interface Test1 {}
export interface Test2 {}
export interface Test3 {}
export interface Test4 {}
// @Filename: file2.ts
import { Test1, Test4 } from './file1';
interface Testing {
    test1: Test1;
    test2: Test2;
    test3: Test3;
    test4: Test4;
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "file2.ts");
    fourslash::unsupported("VerifyCodeFixAll"); // f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}
