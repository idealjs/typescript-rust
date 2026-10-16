use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_inside_object_literal_expression_with_instantiated_class_type() {
    let content = r#"class C1 {
    public a: string;
    protected b: string;
    private c: string;

    constructor(a: string, b = "", c = "") {
        this.a = a;
        this.b = b;
        this.c = c;
    }
}
class C2 {
    public a: string;
    constructor(a: string) {
        this.a = a;
    }
}
function f1(foo: C1 | C2 | { d: number }) {}
f1({ /*1*/ });
function f2(foo: C1 | C2) {}
f2({ /*2*/ });

function f3(foo: C2) {}
f3({ /*3*/ });"#;
    let mut s = Session::new_for_test("completionInsideObjectLiteralExpressionWithInstantiatedClassType", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["a", "d"]);
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["a"]);
    fourslash::verify_completions_exact_at(&mut s, Some("3"), &["a"]);
}
