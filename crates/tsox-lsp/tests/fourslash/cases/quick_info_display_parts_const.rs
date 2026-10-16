use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_display_parts_const() {
    let content = r#"const /*1*/a = 10;
function foo() {
    const /*2*/b = /*3*/a;
    if (b) {
        const /*4*/b1 = 10;
    }
}
namespace m {
    const /*5*/c = 10;
    export const /*6*/d = 10;
    if (c) {
        const /*7*/e = 10;
    }
}
const /*8*/f: () => number = () => 10;
const /*9*/g = /*10*/f;
/*11*/f();
const /*12*/h: { (a: string): number; (a: number): string; } = a => a;
const /*13*/i = /*14*/h;
/*15*/h(10);
/*16*/h("hello");"#;
    let _s = Session::new_for_test("quickInfoDisplayPartsConst", content);
    // TODO: f.VerifyBaselineHover(t)
}
