use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn format_multiple_function_arguments() {
    let content = r#"
 someRandomFunction({
   prop1: 1,
   prop2: 2
 }, {
   prop3: 3,
   prop4: 4
 }, {
   prop5: 5,
   prop6: 6
 });

 someRandomFunction(
     { prop7: 1, prop8: 2 },
     { prop9: 3, prop10: 4 },
     {
       prop11: 5,
       prop2: 6
     }
 );"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(
        &mut s,
        r#"
someRandomFunction({
    prop1: 1,
    prop2: 2
}, {
    prop3: 3,
    prop4: 4
}, {
    prop5: 5,
    prop6: 6
});

someRandomFunction(
    { prop7: 1, prop8: 2 },
    { prop9: 3, prop10: 4 },
    {
        prop11: 5,
        prop2: 6
    }
);"#,
    );
}
