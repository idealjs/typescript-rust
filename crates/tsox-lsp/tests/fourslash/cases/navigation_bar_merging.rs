use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
#[test]
fn navigation_bar_merging() {
    let content = r#"// @Filename: file1.ts
namespace a {
    function foo() {}
}
namespace b {
    function foo() {}
}
namespace a {
    function bar() {}
}
// @Filename: file2.ts
namespace a {}
function a() {}
// @Filename: file3.ts
namespace a {
    interface A {
        foo: number;
    }
}
namespace a {
    interface A {
        bar: number;
    }
}
// @Filename: file4.ts
namespace A { export var x; }
namespace A.B { export var y; }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
    fourslash::go_to_file(&mut s, "file2.ts");
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
    fourslash::go_to_file(&mut s, "file3.ts");
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
    fourslash::go_to_file(&mut s, "file4.ts");
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
