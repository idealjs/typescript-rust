use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_filter_text1() {
    let content = r#"
class Foo1 {
    #bar: number;
    constructor(bar: number) {
        this.[|b|]/*1*/
    }
}

class Foo5 {
	#bar: number;
	constructor(bar: number) {
		this./*5*/
	}
}

class Foo2 {
    #bar: number;
    constructor(bar: number) {
        this.[|#b|]/*2*/
    }
}

class Foo6 {
    #bar: number;
    constructor(bar: number) {
        this.[|#|]/*6*/
    }
}

class Foo3 {
    #bar: number;
    constructor(bar: number) {
       [|b|]/*3*/
    }
}

class Foo7 {
	#bar: number;
	constructor(bar: number) {
	   /*7*/
	}
}

class Foo4 {
    #bar: number;
    constructor(bar: number) {
       [|#b|]/*4*/
    }
}

class Foo8 {
    #bar: number;
    constructor(bar: number) {
       [|#|]/*8*/
    }
}
"#;
    let mut s = Session::new_for_test("completionFilterText1", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "5");
    // TODO: f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "6");
    // TODO: f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "7");
    // TODO: f.VerifyCompletions(t, "7", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "4");
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "8");
    // TODO: f.VerifyCompletions(t, "8", &fourslash.CompletionsExpectedList{
}
