use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_all() {
    let content = r#"// @module: commonjs
// @esModuleInterop: false
// @allowSyntheticDefaultImports: false
// @Filename: /a.ts
export default function ad() {}
export const a0 = 0;
// @Filename: /b.ts
export default function bd() {}
export const b0 = 0;
// @Filename: /c.ts
export default function cd() {}
export const c0 = 0;
// @Filename: /d.ts
export default function dd() {}
export const d0 = 0;
export const d1 = 1;
// @Filename: /e.d.ts
declare function e(): void;
export = e;
// @Filename: /disposable.d.ts
export declare class Disposable { }
// @Filename: /disposable_global.d.ts
interface Disposable { }
// @Filename: /user.ts
import * as b from "./b";
import { } from "./c";
import dd from "./d";

ad; ad; a0; a0;
bd; bd; b0; b0;
cd; cd; c0; c0;
dd; dd; d0; d0; d1; d1;
e; e;
class X extends Disposable { }"#;
    let mut s = Session::new_for_test("importNameCodeFix_all", content);
    fourslash::go_to_file(&mut s, "/user.ts");
    // TODO: f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}
