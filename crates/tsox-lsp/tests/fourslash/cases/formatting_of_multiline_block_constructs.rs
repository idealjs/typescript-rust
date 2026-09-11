use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_of_multiline_block_constructs() {
    let content = r#"namespace InternalModule/*1*/
{
}
interface MyInterface/*2*/
{
}
enum E/*3*/
{
}
class MyClass/*4*/
{
constructor()/*cons*/
{ }
        public MyFunction()/*5*/
        {
                return 0;
        }
public get Getter()/*6*/
{
}
public set Setter(x)/*7*/
{
}
}
function foo()/*8*/
{
{}/*9*/
}
(function()/*10*/
{
});
(() =>/*11*/
{
});
var x :/*12*/
{};/*13*/"#;
    let mut s = Session::new_for_test("formattingOfMultilineBlockConstructs", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"namespace InternalModule {"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"interface MyInterface {"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"enum E {"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"class MyClass {"#);
    fourslash::go_to_marker(&mut s, "cons");
    fourslash::verify_current_line_content(&mut s, r#"    constructor() { }"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"    public MyFunction() {"#);
    fourslash::go_to_marker(&mut s, "6");
    fourslash::verify_current_line_content(&mut s, r#"    public get Getter() {"#);
    fourslash::go_to_marker(&mut s, "7");
    fourslash::verify_current_line_content(&mut s, r#"    public set Setter(x) {"#);
    fourslash::go_to_marker(&mut s, "8");
    fourslash::verify_current_line_content(&mut s, r#"function foo() {"#);
    fourslash::go_to_marker(&mut s, "9");
    fourslash::verify_current_line_content(&mut s, r#"    { }"#);
    fourslash::go_to_marker(&mut s, "10");
    fourslash::verify_current_line_content(&mut s, r#"(function() {"#);
    fourslash::go_to_marker(&mut s, "11");
    fourslash::verify_current_line_content(&mut s, r#"(() => {"#);
    fourslash::go_to_marker(&mut s, "12");
    fourslash::verify_current_line_content(&mut s, r#"var x:"#);
    fourslash::go_to_marker(&mut s, "13");
    fourslash::verify_current_line_content(&mut s, r#"    {};"#);
    // TODO: }
}
