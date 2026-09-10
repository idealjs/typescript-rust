use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_jsx1() {
    let content = r#"// @jsx: react
// @Filename: /node_modules/react/index.d.ts
export const React: any;
// @Filename: /a.tsx
[|<this>|]</this>
// @Filename: /Foo.tsx
export const Foo = 0;
// @Filename: /c.tsx
import { React } from "react";
<Foo />;
// @Filename: /d.tsx
import { Foo } from "./Foo";
<Foo />;"#;
    let mut s = Session::new_for_test("importNameCodeFix_jsx1", content);
    fourslash::go_to_file(&mut s, "/a.tsx");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{}, nil /*preferences*/)
    fourslash::go_to_file(&mut s, "/c.tsx");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
    fourslash::go_to_file(&mut s, "/d.tsx");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
