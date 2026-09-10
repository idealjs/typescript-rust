use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn comments_interface_fourslash() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @lib: es5
/** this is interface 1*/
interface i/*1*/1 {
}
var i1/*2*/_i: i1;
interface nc_/*3*/i1 {
}
var nc_/*4*/i1_i: nc_i1;
/** this is interface 2 with members*/
interface i/*5*/2 {
    /** this is x*/
    x: number;
    /** this is foo*/
    foo: (/**param help*/b: number) => string;
    /** this is indexer*/
    [/**string param*/i: string]: number;
    /**new method*/
    new (/** param*/i: i1);
    nc_x: number;
    nc_foo: (b: number) => string;
    [i: number]: number;
    /** this is call signature*/
    (/**paramhelp a*/a: number,/**paramhelp b*/ b: number) : number;
    /** this is fnfoo*/
    fnfoo(/**param help*/b: number): string;
    nc_fnfoo(b: number): string;
}
var i2/*6*/_i: /*34i*/i2;
var i2_i/*7*/_x = i2_i./*8*/x;
var i2_i/*9*/_foo = i2_i.f/*10*/oo;
var i2_i_f/*11*/oo_r = i2_i.f/*12q*/oo(/*12*/30);
var i2_i_i2_/*13*/si = i2/*13q*/_i["hello"];
var i2_i_i2/*14*/_ii = i2/*14q*/_i[30];
var i2_/*15*/i_n = new i2/*16q*/_i(/*16*/i1_i);
var i2_i/*17*/_nc_x = i2_i.n/*18*/c_x;
var i2_i_/*19*/nc_foo = i2_i.n/*20*/c_foo;
var i2_i_nc_f/*21*/oo_r = i2_i.nc/*22q*/_foo(/*22*/30);
var i2/*23*/_i_r = i2/*24q*/_i(/*24*/10, /*25*/20);
var i2_i/*26*/_fnfoo = i2_i.fn/*27*/foo;
var i2_i_/*28*/fnfoo_r = i2_i.fn/*29q*/foo(/*29*/10);
var i2_i/*30*/_nc_fnfoo = i2_i.nc_fn/*31*/foo;
var i2_i_nc_/*32*/fnfoo_r = i2_i.nc/*33q*/_fnfoo(/*33*/10);
/*34*/
interface i3 {
    /** Comment i3 x*/
    x: number;
    /** Function i3 f*/
    f(/**number parameter*/a: number): string;
    /** i3 l*/
    l: (/**comment i3 l b*/b: number) => string;
    nc_x: number;
    nc_f(a: number): string;
    nc_l: (b: number) => string;
}
var i3_i: i3;
i3_i = {
    /*35*/f: /**own f*/ (/**i3_i a*/a: number) => "Hello" + /*36*/a,
    l: this./*37*/f,
    /** own x*/
    x: this.f(/*38*/10),
    nc_x: this.l(/*39*/this.x),
    nc_f: this.f,
    nc_l: this.l
};
/*40*/i/*40q*/3_i./*41*/f(/*42*/10);
i3_i./*43q*/l(/*43*/10);
i3_i.nc_/*44q*/f(/*44*/10);
i3_i.nc/*45q*/_l(/*45*/10);"#;
    let mut s = Session::new_for_test("commentsInterfaceFourslash", content);
    fourslash::verify_quick_info_at(&mut s, "1", "interface i1", "this is interface 1");
    fourslash::verify_quick_info_at(&mut s, "2", "var i1_i: i1", "");
    fourslash::verify_quick_info_at(&mut s, "3", "interface nc_i1", "");
    fourslash::verify_quick_info_at(&mut s, "4", "var nc_i1_i: nc_i1", "");
    fourslash::verify_quick_info_at(&mut s, "5", "interface i2", "this is interface 2 with members");
    fourslash::verify_quick_info_at(&mut s, "6", "var i2_i: i2", "");
    fourslash::verify_quick_info_at(&mut s, "7", "var i2_i_x: number", "");
    fourslash::verify_quick_info_at(&mut s, "8", "(property) i2.x: number", "this is x");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "8", &fourslash.CompletionsExpectedList{
    fourslash::verify_quick_info_at(&mut s, "9", "var i2_i_foo: (b: number) => string", "");
    fourslash::verify_quick_info_at(&mut s, "10", "(property) i2.foo: (b: number) => string", "this is foo");
    fourslash::verify_quick_info_at(&mut s, "11", "var i2_i_foo_r: string", "");
    fourslash::go_to_marker(&mut s, "12");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "", ParameterDocComment: "
    fourslash::verify_quick_info_at(&mut s, "12q", "(property) i2.foo: (b: number) => string", "this is foo");
    fourslash::verify_quick_info_at(&mut s, "13", "var i2_i_i2_si: number", "");
    fourslash::verify_quick_info_at(&mut s, "13q", "var i2_i: i2", "");
    fourslash::verify_quick_info_at(&mut s, "14", "var i2_i_i2_ii: number", "");
    fourslash::verify_quick_info_at(&mut s, "14q", "var i2_i: i2", "");
    fourslash::verify_quick_info_at(&mut s, "15", "var i2_i_n: any", "");
    fourslash::go_to_marker(&mut s, "16");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "new method", ParameterDoc
    fourslash::verify_quick_info_at(&mut s, "16q", "var i2_i: i2\nnew (i: i1) => any", "new method");
    fourslash::verify_quick_info_at(&mut s, "17", "var i2_i_nc_x: number", "");
    fourslash::verify_quick_info_at(&mut s, "18", "(property) i2.nc_x: number", "");
    fourslash::verify_quick_info_at(&mut s, "19", "var i2_i_nc_foo: (b: number) => string", "");
    fourslash::verify_quick_info_at(&mut s, "20", "(property) i2.nc_foo: (b: number) => string", "");
    fourslash::verify_quick_info_at(&mut s, "21", "var i2_i_nc_foo_r: string", "");
    fourslash::go_to_marker(&mut s, "22");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::verify_quick_info_at(&mut s, "22q", "(property) i2.nc_foo: (b: number) => string", "");
    fourslash::verify_quick_info_at(&mut s, "23", "var i2_i_r: number", "");
    fourslash::go_to_marker(&mut s, "24");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "this is call signature", 
    fourslash::verify_quick_info_at(&mut s, "24q", "var i2_i: i2\n(a: number, b: number) => number", "this is call signature");
    fourslash::go_to_marker(&mut s, "25");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "this is call signature", 
    fourslash::verify_quick_info_at(&mut s, "26", "var i2_i_fnfoo: (b: number) => string", "");
    fourslash::verify_quick_info_at(&mut s, "27", "(method) i2.fnfoo(b: number): string", "this is fnfoo");
    fourslash::verify_quick_info_at(&mut s, "28", "var i2_i_fnfoo_r: string", "");
    fourslash::go_to_marker(&mut s, "29");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "this is fnfoo", Parameter
    fourslash::verify_quick_info_at(&mut s, "29q", "(method) i2.fnfoo(b: number): string", "this is fnfoo");
    fourslash::verify_quick_info_at(&mut s, "30", "var i2_i_nc_fnfoo: (b: number) => string", "");
    fourslash::verify_quick_info_at(&mut s, "31", "(method) i2.nc_fnfoo(b: number): string", "");
    fourslash::verify_quick_info_at(&mut s, "32", "var i2_i_nc_fnfoo_r: string", "");
    fourslash::go_to_marker(&mut s, "33");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::verify_quick_info_at(&mut s, "33q", "(method) i2.nc_fnfoo(b: number): string", "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "34", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "34i", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "36", &fourslash.CompletionsExpectedList{
    fourslash::verify_quick_info_at(&mut s, "40q", "var i3_i: i3", "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "40", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "41");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "(method) i3.f(a: number): string", "Function i3 f")
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "41", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "42");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "Function i3 f", Parameter
    fourslash::go_to_marker(&mut s, "43");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "", ParameterDocComment: "
    fourslash::verify_quick_info_at(&mut s, "43q", "(property) i3.l: (b: number) => string", "i3 l");
    fourslash::go_to_marker(&mut s, "44");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::verify_quick_info_at(&mut s, "44q", "(method) i3.nc_f(a: number): string", "");
    fourslash::go_to_marker(&mut s, "45");
    fourslash::unsupported("VerifySignatureHelp"); // f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::verify_quick_info_at(&mut s, "45q", "(property) i3.nc_l: (b: number) => string", "");
}
