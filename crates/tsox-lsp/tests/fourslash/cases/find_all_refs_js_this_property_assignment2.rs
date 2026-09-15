use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_js_this_property_assignment2() {
    let content = r#"// @allowJs: true
// @noImplicitThis: true
// @Filename: infer.d.ts
export declare function infer(o: { m: Record<string, Function> } & ThisType<{ x: number }>): void;
// @Filename: a.js
import { infer } from "./infer";
infer({
    m: {
        initData() {
            this.x = 1;
            this./*1*/x;
        },
    }
});
// @Filename: b.ts
import { infer } from "./infer";
infer({
    m: {
        initData() {
            this.x = 1;
            this./*2*/x;
        },
    }
});"#;
    let _s = Session::new_for_test("findAllRefsJsThisPropertyAssignment2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2")
}
