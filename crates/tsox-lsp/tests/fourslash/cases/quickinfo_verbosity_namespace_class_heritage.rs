use tsox_lsp::fourslash::Session;


#[test]
fn quickinfo_verbosity_namespace_class_heritage() {
    let content = r#"
declare class Base {
    id: number;
}

declare namespace Shapes/*1*/ {
    class Circle extends Base {
        radius: number;
    }
    class Square extends Base {
        side: number;
    }
    interface Drawable {
        draw(): void;
    }
}
"#;
    let _s = Session::new_for_test("quickinfoVerbosityNamespaceClassHeritage", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
