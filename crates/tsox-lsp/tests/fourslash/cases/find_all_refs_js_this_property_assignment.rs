use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_js_this_property_assignment() {
    let content = r#"// @allowJs: true
// @noImplicitThis: true
// @Filename: infer.d.ts
export declare function infer(o: { m(): void } & ThisType<{ x: number }>): void;
// @Filename: a.js
import { infer } from "./infer";
infer({
    m() {
        this.x = 1;
        this./*1*/x;
    },
});
// @Filename: b.js
/**
 * @template T
 * @param {{m(): void} & ThisType<{x: number}>} o
 */
function infer(o) {}
infer({
    m() {
        this.x = 2;
        this./*2*/x;
    },
});"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
