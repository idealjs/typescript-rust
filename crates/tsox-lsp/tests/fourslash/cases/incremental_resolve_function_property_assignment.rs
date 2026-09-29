use tsox_lsp::fourslash::{self, Session};


#[test]
fn incremental_resolve_function_property_assignment() {
    let content = r#"function bar(indexer: { getLength(): number; getTypeAtIndex(index: number): string; }): string {
    return indexer.getTypeAtIndex(indexer.getLength() - 1);
}
function foo(a: string[]) {
    return bar({
        getLength(): number {
            return "a.length";
        },
        getTypeAtIndex(index: number) {
            switch (index) {
                case 0: return a[0];
                case 1: return a[1];
                case 2: return a[2];
                default: return "invalid";
            }
        }
    });
}
var val = foo(["myString1", "myString2"]);
/*1*/val;"#;
    let mut s = Session::new_for_test("incrementalResolveFunctionPropertyAssignment", content);
    fourslash::verify_quick_info_at(&mut s, "1", "var val: string", "");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
