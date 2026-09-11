use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn comments_inheritance_fourslash() {
    let content = r#"/** i1 is interface with properties*/
interface i1 {
    /** i1_p1*/
    i1_p1: number;
    /** i1_f1*/
    i1_f1(): void;
    /** i1_l1*/
    i1_l1: () => void;
    i1_nc_p1: number;
    i1_nc_f1(): void;
    i1_nc_l1: () => void;
    p1: number;
    f1(): void;
    l1: () => void;
    nc_p1: number;
    nc_f1(): void;
    nc_l1: () => void;
}
class c1 implements i1 {
    public i1_p1: number;
    public i1_f1() {
    }
    public i1_l1: () => void;
    public i1_nc_p1: number;
    public i1_nc_f1() {
    }
    public i1_nc_l1: () => void;
    /** c1_p1*/
    public p1: number;
    /** c1_f1*/
    public f1() {
    }
    /** c1_l1*/
    public l1: () => void;
    /** c1_nc_p1*/
    public nc_p1: number;
    /** c1_nc_f1*/
    public nc_f1() {
    }
    /** c1_nc_l1*/
    public nc_l1: () => void;
}
var i1/*1iq*/_i: /*16i*/i1;
i1_i./*1*/i/*2q*/1_f1(/*2*/);
i1_i.i1_n/*3q*/c_f1(/*3*/);
i1_i.f/*4q*/1(/*4*/);
i1_i.nc/*5q*/_f1(/*5*/);
i1_i.i1/*l2q*/_l1(/*l2*/);
i1_i.i1_/*l3q*/nc_l1(/*l3*/);
i1_i.l/*l4q*/1(/*l4*/);
i1_i.nc/*l5q*/_l1(/*l5*/);
var c1/*6iq*/_i = new c1();
c1_i./*6*/i1/*7q*/_f1(/*7*/);
c1_i.i1_nc/*8q*/_f1(/*8*/);
c1_i.f/*9q*/1(/*9*/);
c1_i.nc/*10q*/_f1(/*10*/);
c1_i.i1/*l7q*/_l1(/*l7*/);
c1_i.i1_n/*l8q*/c_l1(/*l8*/);
c1_i.l/*l9q*/1(/*l9*/);
c1_i.nc/*l10q*/_l1(/*l10*/);
// assign to interface
i1_i = c1_i;
i1_i./*11*/i1/*12q*/_f1(/*12*/);
i1_i.i1_nc/*13q*/_f1(/*13*/);
i1_i.f/*14q*/1(/*14*/);
i1_i.nc/*15q*/_f1(/*15*/);
i1_i.i1/*l12q*/_l1(/*l12*/);
i1_i.i1/*l13q*/_nc_l1(/*l13*/);
i1_i.l/*l14q*/1(/*l14*/);
i1_i.nc/*l15q*/_l1(/*l15*/);
/*16*/
class c2 {
    /** c2 c2_p1*/
    public c2_p1: number;
    /** c2 c2_f1*/
    public c2_f1() {
    }
    /** c2 c2_prop*/
    public get c2_prop() {
        return 10;
    }
    public c2_nc_p1: number;
    public c2_nc_f1() {
    }
    public get c2_nc_prop() {
        return 10;
    }
    /** c2 p1*/
    public p1: number;
    /** c2 f1*/
    public f1() {
    }
    /** c2 prop*/
    public get prop() {
        return 10;
    }
    public nc_p1: number;
    public nc_f1() {
    }
    public get nc_prop() {
        return 10;
    }
    /** c2 constructor*/
    constr/*55*/uctor(a: number) {
        this.c2_p1 = a;
    }
}
class c3 extends c2 {
    cons/*56*/tructor() {
        su/*18sq*/per(10);
        this.p1 = s/*18spropq*/uper./*18spropProp*/c2_p1;
    }
    /** c3 p1*/
    public p1: number;
    /** c3 f1*/
    public f1() {
    }
    /** c3 prop*/
    public get prop() {
        return 10;
    }
    public nc_p1: number;
    public nc_f1() {
    }
    public get nc_prop() {
        return 10;
    }
}
var c/*17iq*/2_i = new c/*17q*/2(/*17*/10);
var c/*18iq*/3_i = new c/*18q*/3(/*18*/);
c2_i./*19*/c2/*20q*/_f1(/*20*/);
c2_i.c2_nc/*21q*/_f1(/*21*/);
c2_i.f/*22q*/1(/*22*/);
c2_i.nc/*23q*/_f1(/*23*/);
c3_i./*24*/c2/*25q*/_f1(/*25*/);
c3_i.c2_nc/*26q*/_f1(/*26*/);
c3_i.f/*27q*/1(/*27*/);
c3_i.nc/*28q*/_f1(/*28*/);
// assign
c2_i = c3_i;
c2_i./*29*/c2/*30q*/_f1(/*30*/);
c2_i.c2_nc_/*31q*/f1(/*31*/);
c2_i.f/*32q*/1(/*32*/);
c2_i.nc/*33q*/_f1(/*33*/);
class c4 extends c2 {
}
var c4/*34iq*/_i = new c/*34q*/4(/*34*/10);
/*35*/
interface i2 {
    /** i2_p1*/
    i2_p1: number;
    /** i2_f1*/
    i2_f1(): void;
    /** i2_l1*/
    i2_l1: () => void;
    i2_nc_p1: number;
    i2_nc_f1(): void;
    i2_nc_l1: () => void;
    /** i2 p1*/
    p1: number;
    /** i2 f1*/
    f1(): void;
    /** i2 l1*/
    l1: () => void;
    nc_p1: number;
    nc_f1(): void;
    nc_l1: () => void;
}
interface i3 extends i2 {
    /** i3 p1*/
    p1: number;
    /** i3 f1*/
    f1(): void;
    /** i3 l1*/
    l1: () => void;
    nc_p1: number;
    nc_f1(): void;
    nc_l1: () => void;
}
var i2/*36iq*/_i: /*51i*/i2;
var i3/*37iq*/_i: i3;
i2_i./*36*/i2/*37q*/_f1(/*37*/);
i2_i.i2_n/*38q*/c_f1(/*38*/);
i2_i.f/*39q*/1(/*39*/);
i2_i.nc/*40q*/_f1(/*40*/);
i2_i.i2_/*l37q*/l1(/*l37*/);
i2_i.i2_nc/*l38q*/_l1(/*l38*/);
i2_i.l/*l39q*/1(/*l39*/);
i2_i.nc_/*l40q*/l1(/*l40*/);
i3_i./*41*/i2_/*42q*/f1(/*42*/);
i3_i.i2_nc/*43q*/_f1(/*43*/);
i3_i.f/*44q*/1(/*44*/);
i3_i.nc_/*45q*/f1(/*45*/);
i3_i.i2_/*l42q*/l1(/*l42*/);
i3_i.i2_nc/*l43q*/_l1(/*l43*/);
i3_i.l/*l44q*/1(/*l44*/);
i3_i.nc_/*l45q*/l1(/*l45*/);
// assign to interface
i2_i = i3_i;
i2_i./*46*/i2/*47q*/_f1(/*47*/);
i2_i.i2_nc_/*48q*/f1(/*48*/);
i2_i.f/*49q*/1(/*49*/);
i2_i.nc/*50q*/_f1(/*50*/);
i2_i.i2_/*l47q*/l1(/*l47*/);
i2_i.i2_nc/*l48q*/_l1(/*l48*/);
i2_i.l/*l49q*/1(/*l49*/);
i2_i.nc_/*l50q*/l1(/*l50*/);
/*51*/
/**c5 class*/
class c5 {
    public b: number;
}
class c6 extends c5 {
    public d;
    const/*57*/ructor() {
        /*52*/super();
        this.d = /*53*/super./*54*/b;
    }
}"#;
    let mut s = Session::new_for_test("commentsInheritanceFourslash", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "11"}, &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "i1_f1"})
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "4");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "5");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l2");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l3");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l4");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l5");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::verify_quick_info_at(&mut s, "1iq", "var i1_i: i1", "");
    fourslash::verify_quick_info_at(&mut s, "2q", "(method) i1.i1_f1(): void", "i1_f1");
    fourslash::verify_quick_info_at(&mut s, "3q", "(method) i1.i1_nc_f1(): void", "");
    fourslash::verify_quick_info_at(&mut s, "4q", "(method) i1.f1(): void", "");
    fourslash::verify_quick_info_at(&mut s, "5q", "(method) i1.nc_f1(): void", "");
    fourslash::verify_quick_info_at(&mut s, "l2q", "(property) i1.i1_l1: () => void", "i1_l1");
    fourslash::verify_quick_info_at(&mut s, "l3q", "(property) i1.i1_nc_l1: () => void", "");
    fourslash::verify_quick_info_at(&mut s, "l4q", "(property) i1.l1: () => void", "");
    fourslash::verify_quick_info_at(&mut s, "l5q", "(property) i1.nc_l1: () => void", "");
    // TODO: f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "7");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "i1_f1"})
    fourslash::go_to_marker(&mut s, "9");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "c1_f1"})
    fourslash::go_to_marker(&mut s, "10");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "c1_nc_f1"})
    fourslash::go_to_marker(&mut s, "l9");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "c1_l1"})
    fourslash::go_to_marker(&mut s, "l10");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "c1_nc_l1"})
    fourslash::go_to_marker(&mut s, "8");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l7");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l8");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::verify_quick_info_at(&mut s, "6iq", "var c1_i: c1", "");
    fourslash::verify_quick_info_at(&mut s, "7q", "(method) c1.i1_f1(): void", "i1_f1");
    fourslash::verify_quick_info_at(&mut s, "8q", "(method) c1.i1_nc_f1(): void", "");
    fourslash::verify_quick_info_at(&mut s, "9q", "(method) c1.f1(): void", "c1_f1");
    fourslash::verify_quick_info_at(&mut s, "10q", "(method) c1.nc_f1(): void", "c1_nc_f1");
    fourslash::verify_quick_info_at(&mut s, "l7q", "(property) c1.i1_l1: () => void", "i1_l1");
    fourslash::verify_quick_info_at(&mut s, "l8q", "(property) c1.i1_nc_l1: () => void", "");
    fourslash::verify_quick_info_at(&mut s, "l9q", "(property) c1.l1: () => void", "c1_l1");
    fourslash::verify_quick_info_at(&mut s, "l10q", "(property) c1.nc_l1: () => void", "c1_nc_l1");
    // TODO: f.VerifyCompletions(t, "11", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "12");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "i1_f1"})
    fourslash::go_to_marker(&mut s, "13");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "14");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "15");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l12");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l13");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l14");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l15");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::verify_quick_info_at(&mut s, "12q", "(method) i1.i1_f1(): void", "i1_f1");
    fourslash::verify_quick_info_at(&mut s, "13q", "(method) i1.i1_nc_f1(): void", "");
    fourslash::verify_quick_info_at(&mut s, "14q", "(method) i1.f1(): void", "");
    fourslash::verify_quick_info_at(&mut s, "15q", "(method) i1.nc_f1(): void", "");
    fourslash::verify_quick_info_at(&mut s, "l12q", "(property) i1.i1_l1: () => void", "i1_l1");
    fourslash::verify_quick_info_at(&mut s, "l13q", "(property) i1.i1_nc_l1: () => void", "");
    fourslash::verify_quick_info_at(&mut s, "l14q", "(property) i1.l1: () => void", "");
    fourslash::verify_quick_info_at(&mut s, "l15q", "(property) i1.nc_l1: () => void", "");
    // TODO: f.VerifyCompletions(t, "16", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "16i", &fourslash.CompletionsExpectedList{
    fourslash::verify_quick_info_at(&mut s, "17iq", "var c2_i: c2", "");
    fourslash::verify_quick_info_at(&mut s, "18iq", "var c3_i: c3", "");
    fourslash::go_to_marker(&mut s, "17");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "c2 constructor"})
    fourslash::go_to_marker(&mut s, "18");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::verify_quick_info_at(&mut s, "18sq", "constructor c2(a: number): c2", "c2 constructor");
    fourslash::verify_quick_info_at(&mut s, "18spropq", "class c2", "");
    fourslash::verify_quick_info_at(&mut s, "18spropProp", "(property) c2.c2_p1: number", "c2 c2_p1");
    fourslash::verify_quick_info_at(&mut s, "17q", "constructor c2(a: number): c2", "c2 constructor");
    fourslash::verify_quick_info_at(&mut s, "18q", "constructor c3(): c3", "");
    // TODO: f.VerifyCompletions(t, []string{"19", "29"}, &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "20");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "c2 c2_f1"})
    fourslash::go_to_marker(&mut s, "22");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "c2 f1"})
    fourslash::go_to_marker(&mut s, "21");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "23");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::verify_quick_info_at(&mut s, "20q", "(method) c2.c2_f1(): void", "c2 c2_f1");
    fourslash::verify_quick_info_at(&mut s, "21q", "(method) c2.c2_nc_f1(): void", "");
    fourslash::verify_quick_info_at(&mut s, "22q", "(method) c2.f1(): void", "c2 f1");
    fourslash::verify_quick_info_at(&mut s, "23q", "(method) c2.nc_f1(): void", "");
    // TODO: f.VerifyCompletions(t, "24", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "25");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "c2 c2_f1"})
    fourslash::go_to_marker(&mut s, "27");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "c3 f1"})
    fourslash::go_to_marker(&mut s, "26");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "28");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::verify_quick_info_at(&mut s, "25q", "(method) c2.c2_f1(): void", "c2 c2_f1");
    fourslash::verify_quick_info_at(&mut s, "26q", "(method) c2.c2_nc_f1(): void", "");
    fourslash::verify_quick_info_at(&mut s, "27q", "(method) c3.f1(): void", "c3 f1");
    fourslash::verify_quick_info_at(&mut s, "28q", "(method) c3.nc_f1(): void", "");
    fourslash::go_to_marker(&mut s, "30");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "c2 c2_f1"})
    fourslash::go_to_marker(&mut s, "32");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "c2 f1"})
    fourslash::go_to_marker(&mut s, "31");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "33");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::verify_quick_info_at(&mut s, "30q", "(method) c2.c2_f1(): void", "c2 c2_f1");
    fourslash::verify_quick_info_at(&mut s, "31q", "(method) c2.c2_nc_f1(): void", "");
    fourslash::verify_quick_info_at(&mut s, "32q", "(method) c2.f1(): void", "c2 f1");
    fourslash::verify_quick_info_at(&mut s, "33q", "(method) c2.nc_f1(): void", "");
    fourslash::go_to_marker(&mut s, "34");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "c2 constructor"})
    fourslash::verify_quick_info_at(&mut s, "34iq", "var c4_i: c4", "");
    fourslash::verify_quick_info_at(&mut s, "34q", "constructor c4(a: number): c4", "c2 constructor");
    // TODO: f.VerifyCompletions(t, "35", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"36", "46"}, &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "37");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "i2_f1"})
    fourslash::go_to_marker(&mut s, "39");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "i2 f1"})
    fourslash::go_to_marker(&mut s, "38");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "40");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l37");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l37");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l39");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l40");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::verify_quick_info_at(&mut s, "36iq", "var i2_i: i2", "");
    fourslash::verify_quick_info_at(&mut s, "37iq", "var i3_i: i3", "");
    fourslash::verify_quick_info_at(&mut s, "37q", "(method) i2.i2_f1(): void", "i2_f1");
    fourslash::verify_quick_info_at(&mut s, "38q", "(method) i2.i2_nc_f1(): void", "");
    fourslash::verify_quick_info_at(&mut s, "39q", "(method) i2.f1(): void", "i2 f1");
    fourslash::verify_quick_info_at(&mut s, "40q", "(method) i2.nc_f1(): void", "");
    fourslash::verify_quick_info_at(&mut s, "l37q", "(property) i2.i2_l1: () => void", "i2_l1");
    fourslash::verify_quick_info_at(&mut s, "l38q", "(property) i2.i2_nc_l1: () => void", "");
    fourslash::verify_quick_info_at(&mut s, "l39q", "(property) i2.l1: () => void", "i2 l1");
    fourslash::verify_quick_info_at(&mut s, "l40q", "(property) i2.nc_l1: () => void", "");
    // TODO: f.VerifyCompletions(t, "41", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "42");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "i2_f1"})
    fourslash::go_to_marker(&mut s, "44");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "i3 f1"})
    fourslash::go_to_marker(&mut s, "43");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "45");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l42");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l43");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l44");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l45");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::verify_quick_info_at(&mut s, "42q", "(method) i2.i2_f1(): void", "i2_f1");
    fourslash::verify_quick_info_at(&mut s, "43q", "(method) i2.i2_nc_f1(): void", "");
    fourslash::verify_quick_info_at(&mut s, "44q", "(method) i3.f1(): void", "i3 f1");
    fourslash::verify_quick_info_at(&mut s, "45q", "(method) i3.nc_f1(): void", "");
    fourslash::verify_quick_info_at(&mut s, "l42q", "(property) i2.i2_l1: () => void", "i2_l1");
    fourslash::verify_quick_info_at(&mut s, "l43q", "(property) i2.i2_nc_l1: () => void", "");
    fourslash::verify_quick_info_at(&mut s, "l44q", "(property) i3.l1: () => void", "i3 l1");
    fourslash::verify_quick_info_at(&mut s, "l45q", "(property) i3.nc_l1: () => void", "");
    // TODO: f.VerifyCompletions(t, "46", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "47");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "i2_f1"})
    fourslash::go_to_marker(&mut s, "49");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: "i2 f1"})
    fourslash::go_to_marker(&mut s, "48");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l47");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l48");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l49");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::go_to_marker(&mut s, "l50");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{DocComment: ""})
    fourslash::verify_quick_info_at(&mut s, "47q", "(method) i2.i2_f1(): void", "i2_f1");
    fourslash::verify_quick_info_at(&mut s, "48q", "(method) i2.i2_nc_f1(): void", "");
    fourslash::verify_quick_info_at(&mut s, "49q", "(method) i2.f1(): void", "i2 f1");
    fourslash::verify_quick_info_at(&mut s, "50q", "(method) i2.nc_f1(): void", "");
    fourslash::verify_quick_info_at(&mut s, "l47q", "(property) i2.i2_l1: () => void", "i2_l1");
    fourslash::verify_quick_info_at(&mut s, "l48q", "(property) i2.i2_nc_l1: () => void", "");
    fourslash::verify_quick_info_at(&mut s, "l49q", "(property) i2.l1: () => void", "i2 l1");
    fourslash::verify_quick_info_at(&mut s, "l40q", "(property) i2.nc_l1: () => void", "");
    // TODO: f.VerifyCompletions(t, "51", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "51i", &fourslash.CompletionsExpectedList{
    fourslash::verify_quick_info_at(&mut s, "52", "constructor c5(): c5", "c5 class");
    fourslash::verify_quick_info_at(&mut s, "53", "class c5", "c5 class");
    fourslash::verify_quick_info_at(&mut s, "54", "(property) c5.b: number", "");
    fourslash::verify_quick_info_at(&mut s, "55", "constructor c2(a: number): c2", "c2 constructor");
    fourslash::verify_quick_info_at(&mut s, "56", "constructor c3(): c3", "");
    fourslash::verify_quick_info_at(&mut s, "57", "constructor c6(): c6", "");
}
