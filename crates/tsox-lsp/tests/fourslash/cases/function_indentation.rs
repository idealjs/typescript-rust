use tsox_lsp::fourslash::{self, Session};


#[test]
fn function_indentation() {
    let content = r#"namespace M {
export =
C;
class C {
constructor(b
) {
}
foo(a
: string) {
return a
|| true;
}
get bar(
) {
return 1;
}
}
function foo(a,
b?) {
new M.C(
"hello");
}
{
{
}
}
foo(
function() {
"hello";
});
foo(
() => {
"hello";
});
var t,
u = 1,
v;
}"#;
    let mut s = Session::new_for_test("functionIndentation", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, concat!(r#"namespace M {
"#, r#"    export =
"#, r#"        C;
"#, r#"    class C {
"#, r#"        constructor(b
"#, r#"        ) {
"#, r#"        }
"#, r#"        foo(a
"#, r#"            : string) {
"#, r#"            return a
"#, r#"                || true;
"#, r#"        }
"#, r#"        get bar(
"#, r#"        ) {
"#, r#"            return 1;
"#, r#"        }
"#, r#"    }
"#, r#"    function foo(a,
"#, r#"        b?) {
"#, r#"        new M.C(
"#, r#"            "hello");
"#, r#"    }
"#, r#"    {
"#, r#"        {
"#, r#"        }
"#, r#"    }
"#, r#"    foo(
"#, r#"        function() {
"#, r#"            "hello";
"#, r#"        });
"#, r#"    foo(
"#, r#"        () => {
"#, r#"            "hello";
"#, r#"        });
"#, r#"    var t,
"#, r#"        u = 1,
"#, r#"        v;
"#, r#"}"#));
}
