use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_namespace_members() {
    let content = r#"
declare namespace NS/*1*/ {
    type StringAlias = string;
    type Pair<T> = { first: T; second: T };

    enum Color { Red, Green, Blue }

    class MyClass {
        name: string;
        greet(): void;
    }

    interface MyInterface {
        id: number;
        label: string;
    }

    const value: number;
    function doSomething(x: string): boolean;
}
"#;
    let mut s = Session::new_for_test("quickinfoVerbosityNamespaceMembers", content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
