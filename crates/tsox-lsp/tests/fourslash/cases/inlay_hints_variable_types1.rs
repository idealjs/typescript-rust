use tsox_lsp::fourslash::{self, Session};


#[test]
fn inlay_hints_variable_types1() {
    let content = r#"class C {}
namespace N { export class Foo {} }
interface Foo {}
const a = "a";
const b = 1;
const c = true;
const d = {} as Foo;
const e = <Foo>{};
const f = {} as const;
const g = (({} as const));
const h = new C();
const i = new N.C();
const j = ((((new C()))));
const k = { a: 1, b: 1 };
const l = ((({ a: 1, b: 1 })));
 const m = () => 123;
 const n;"#;
    let mut s = Session::new_for_test("inlayHintsVariableTypes1", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
