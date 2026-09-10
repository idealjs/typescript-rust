use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn formatting_on_classes() {
    let content = r#"/*1*/         class                    a                  {
/*2*/                                                        constructor       (       n   :                 number    )             ;
/*3*/                                                        constructor       (       s   :                 string    )             ;
/*4*/                                                        constructor       (       ns   :                 any    )                            {

/*5*/                                                        }

/*6*/                                                            public                 pgF       (           )                            {                  }

/*7*/                                                            public                 pv   ;
/*8*/                                                            public                 get              d       (           )                            {
/*9*/                                                                                                                return              30   ;
/*10*/                                                        }
/*11*/                                                            public                 set              d       (       number        )                            {
/*12*/                                                        }

/*13*/                                                            public                 static                    get              p2       (           )                            {
/*14*/                                                                                                                return                  {                  x   :                 30   ,                  y   :                 40              }   ;
/*15*/                                                        }

/*16*/                                                                         private                static                    d2       (           )                            {
/*17*/                                                        }
/*18*/                                                                         private                static                    get              p3       (           )                            {
/*19*/                                                                                                                return              "string"   ;
/*20*/                                                        }
/*21*/                                                                         private                pv3   ;

/*22*/                                                                         private                foo       (       n   :                 number    )             :                 string   ;
/*23*/                                                                         private                foo       (       s   :                 string    )             :                 string   ;
/*24*/                                                                         private                foo       (       ns   :                 any    )                            {
/*25*/                                                                                                                return              ns.toString       (           )             ;
/*26*/                                                        }
/*27*/}

/*28*/         class                    b              extends              a                  {
/*29*/}

/*30*/         class   m1b      {

/*31*/}

/*32*/                                                interface   m1ib                               {

/*33*/  }
/*34*/         class                    c              extends              m1b                  {
/*35*/}

/*36*/         class                    ib2              implements              m1ib                  {
/*37*/}

/*38*/    declare                            class                    aAmbient                  {
/*39*/                                                        constructor                     (       n   :                 number    )             ;
/*40*/                                                        constructor                     (       s   :                 string    )             ;
/*41*/                                                            public                 pgF       (           )             :                 void   ;
/*42*/                                                            public                 pv   ;
/*43*/                                                            public                 d                 :                 number   ;
/*44*/                                                        static                    p2                 :                     {                  x   :                 number   ;              y   :                 number   ;              }   ;
/*45*/                                                        static                    d2       (           )             ;
/*46*/                                                        static                    p3   ;
/*47*/                                                                         private                pv3   ;
/*48*/                                                                         private                foo       (       s    )             ;
/*49*/}

/*50*/         class                    d                  {
/*51*/                                                                         private                foo       (       n   :                 number    )             :                 string   ;
/*52*/                                                                         private                foo       (       s   :                 string    )             :                 string   ;
/*53*/                                                                         private                foo       (       ns   :                 any    )                            {
/*54*/                                                                                                                return              ns.toString       (           )             ;
/*55*/                                                        }
/*56*/}

/*57*/         class                    e                  {
/*58*/                                                                         private                foo       (       s   :                 string    )             :                 string   ;
/*59*/                                                                         private                foo       (       n   :                 number    )             :                 string   ;
/*60*/                                                                         private                foo       (       ns   :                 any    )                            {
/*61*/                                                                                                                return              ns.toString       (           )             ;
/*62*/                                                        }
/*63*/                                                                         protected              bar        (            )  {                 }
/*64*/                                                                         protected     static   bar2       (            )  {                 }
/*65*/                                                                         private                pv4  :    number =
/*66*/                                                                         {};
/*END*/}"#;
    let mut s = Session::new_for_test("formattingOnClasses", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"class a {"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"    constructor(n: number);"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"    constructor(s: string);"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"    constructor(ns: any) {"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
    fourslash::go_to_marker(&mut s, "6");
    fourslash::verify_current_line_content(&mut s, r#"    public pgF() { }"#);
    fourslash::go_to_marker(&mut s, "7");
    fourslash::verify_current_line_content(&mut s, r#"    public pv;"#);
    fourslash::go_to_marker(&mut s, "8");
    fourslash::verify_current_line_content(&mut s, r#"    public get d() {"#);
    fourslash::go_to_marker(&mut s, "9");
    fourslash::verify_current_line_content(&mut s, r#"        return 30;"#);
    fourslash::go_to_marker(&mut s, "10");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
    fourslash::go_to_marker(&mut s, "11");
    fourslash::verify_current_line_content(&mut s, r#"    public set d(number) {"#);
    fourslash::go_to_marker(&mut s, "12");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
    fourslash::go_to_marker(&mut s, "13");
    fourslash::verify_current_line_content(&mut s, r#"    public static get p2() {"#);
    fourslash::go_to_marker(&mut s, "14");
    fourslash::verify_current_line_content(&mut s, r#"        return { x: 30, y: 40 };"#);
    fourslash::go_to_marker(&mut s, "15");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
    fourslash::go_to_marker(&mut s, "16");
    fourslash::verify_current_line_content(&mut s, r#"    private static d2() {"#);
    fourslash::go_to_marker(&mut s, "17");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
    fourslash::go_to_marker(&mut s, "18");
    fourslash::verify_current_line_content(&mut s, r#"    private static get p3() {"#);
    fourslash::go_to_marker(&mut s, "19");
    fourslash::verify_current_line_content(&mut s, r#"        return "string";"#);
    fourslash::go_to_marker(&mut s, "20");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
    fourslash::go_to_marker(&mut s, "21");
    fourslash::verify_current_line_content(&mut s, r#"    private pv3;"#);
    fourslash::go_to_marker(&mut s, "22");
    fourslash::verify_current_line_content(&mut s, r#"    private foo(n: number): string;"#);
    fourslash::go_to_marker(&mut s, "23");
    fourslash::verify_current_line_content(&mut s, r#"    private foo(s: string): string;"#);
    fourslash::go_to_marker(&mut s, "24");
    fourslash::verify_current_line_content(&mut s, r#"    private foo(ns: any) {"#);
    fourslash::go_to_marker(&mut s, "25");
    fourslash::verify_current_line_content(&mut s, r#"        return ns.toString();"#);
    fourslash::go_to_marker(&mut s, "26");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
    fourslash::go_to_marker(&mut s, "27");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "28");
    fourslash::verify_current_line_content(&mut s, r#"class b extends a {"#);
    fourslash::go_to_marker(&mut s, "29");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "30");
    fourslash::verify_current_line_content(&mut s, r#"class m1b {"#);
    fourslash::go_to_marker(&mut s, "31");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "32");
    fourslash::verify_current_line_content(&mut s, r#"interface m1ib {"#);
    fourslash::go_to_marker(&mut s, "33");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "34");
    fourslash::verify_current_line_content(&mut s, r#"class c extends m1b {"#);
    fourslash::go_to_marker(&mut s, "35");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "36");
    fourslash::verify_current_line_content(&mut s, r#"class ib2 implements m1ib {"#);
    fourslash::go_to_marker(&mut s, "37");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "38");
    fourslash::verify_current_line_content(&mut s, r#"declare class aAmbient {"#);
    fourslash::go_to_marker(&mut s, "39");
    fourslash::verify_current_line_content(&mut s, r#"    constructor(n: number);"#);
    fourslash::go_to_marker(&mut s, "40");
    fourslash::verify_current_line_content(&mut s, r#"    constructor(s: string);"#);
    fourslash::go_to_marker(&mut s, "41");
    fourslash::verify_current_line_content(&mut s, r#"    public pgF(): void;"#);
    fourslash::go_to_marker(&mut s, "42");
    fourslash::verify_current_line_content(&mut s, r#"    public pv;"#);
    fourslash::go_to_marker(&mut s, "43");
    fourslash::verify_current_line_content(&mut s, r#"    public d: number;"#);
    fourslash::go_to_marker(&mut s, "44");
    fourslash::verify_current_line_content(&mut s, r#"    static p2: { x: number; y: number; };"#);
    fourslash::go_to_marker(&mut s, "45");
    fourslash::verify_current_line_content(&mut s, r#"    static d2();"#);
    fourslash::go_to_marker(&mut s, "46");
    fourslash::verify_current_line_content(&mut s, r#"    static p3;"#);
    fourslash::go_to_marker(&mut s, "47");
    fourslash::verify_current_line_content(&mut s, r#"    private pv3;"#);
    fourslash::go_to_marker(&mut s, "48");
    fourslash::verify_current_line_content(&mut s, r#"    private foo(s);"#);
    fourslash::go_to_marker(&mut s, "49");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "50");
    fourslash::verify_current_line_content(&mut s, r#"class d {"#);
    fourslash::go_to_marker(&mut s, "51");
    fourslash::verify_current_line_content(&mut s, r#"    private foo(n: number): string;"#);
    fourslash::go_to_marker(&mut s, "52");
    fourslash::verify_current_line_content(&mut s, r#"    private foo(s: string): string;"#);
    fourslash::go_to_marker(&mut s, "53");
    fourslash::verify_current_line_content(&mut s, r#"    private foo(ns: any) {"#);
    fourslash::go_to_marker(&mut s, "54");
    fourslash::verify_current_line_content(&mut s, r#"        return ns.toString();"#);
    fourslash::go_to_marker(&mut s, "55");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
    fourslash::go_to_marker(&mut s, "56");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
    fourslash::go_to_marker(&mut s, "57");
    fourslash::verify_current_line_content(&mut s, r#"class e {"#);
    fourslash::go_to_marker(&mut s, "58");
    fourslash::verify_current_line_content(&mut s, r#"    private foo(s: string): string;"#);
    fourslash::go_to_marker(&mut s, "59");
    fourslash::verify_current_line_content(&mut s, r#"    private foo(n: number): string;"#);
    fourslash::go_to_marker(&mut s, "60");
    fourslash::verify_current_line_content(&mut s, r#"    private foo(ns: any) {"#);
    fourslash::go_to_marker(&mut s, "61");
    fourslash::verify_current_line_content(&mut s, r#"        return ns.toString();"#);
    fourslash::go_to_marker(&mut s, "62");
    fourslash::verify_current_line_content(&mut s, r#"    }"#);
    fourslash::go_to_marker(&mut s, "63");
    fourslash::verify_current_line_content(&mut s, r#"    protected bar() { }"#);
    fourslash::go_to_marker(&mut s, "64");
    fourslash::verify_current_line_content(&mut s, r#"    protected static bar2() { }"#);
    fourslash::go_to_marker(&mut s, "65");
    fourslash::verify_current_line_content(&mut s, r#"    private pv4: number ="#);
    fourslash::go_to_marker(&mut s, "66");
    fourslash::verify_current_line_content(&mut s, r#"        {};"#);
    fourslash::go_to_marker(&mut s, "END");
    fourslash::verify_current_line_content(&mut s, r#"}"#);
}
