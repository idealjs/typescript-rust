use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatting_fat_arrow_functions() {
    let content = r#"// valid
    (         )           =>    1  ;/*1*/
    (        arg )           =>    2  ;/*2*/
        arg       =>    2  ;/*3*/
        arg=>2  ;/*3a*/
      (        arg     = 1 )           =>    3  ;/*4*/
    (        arg    ?        )           =>    4  ;/*5*/
    (        arg    :    number )           =>    5  ;/*6*/
      (        arg    :    number     = 0 )           =>    6  ;/*7*/
    (        arg        ?                  :    number )           =>    7  ;/*8*/
    (                 ...     arg    :    number   [      ]    )           =>    8  ;/*9*/
      (        arg1   ,    arg2 )           =>    12  ;/*10*/
    (        arg1     = 1   ,    arg2     =3 )           =>    13  ;/*11*/
      (        arg1    ?          ,    arg2    ?        )           =>    14  ;/*12*/
    (        arg1    :    number   ,    arg2    :    number )           =>    15  ;/*13*/
    (        arg1    :    number     = 0   ,    arg2    :    number     = 1 )           =>    16  ;/*14*/
      (        arg1    ?           :    number   ,    arg2    ?           :    number )           =>    17  ;/*15*/
    (        arg1   ,             ...     arg2    :    number   [      ]    )           =>    18  ;/*16*/
      (        arg1   ,    arg2    ?           :    number )           =>    19  ;/*17*/

// in paren
    (            (         )           =>    21 )      ;/*18*/
    (            (        arg )           =>    22 )      ;/*19*/
    (            (        arg     = 1 )           =>    23 )      ;/*20*/
    (            (        arg    ?        )           =>    24 )      ;/*21*/
    (            (        arg    :    number )           =>    25 )      ;/*22*/
    (            (        arg    :    number     = 0 )           =>    26 )      ;/*23*/
    (            (        arg    ?           :    number )           =>    27 )      ;/*24*/
    (            (                 ...     arg    :    number   [      ]    )           =>    28 )      ;/*25*/

// in multiple paren
    (            (            (            (            (        arg )           =>    { return 32  ;    } )     )     )     )      ;/*26*/

// in ternary exression
      false        ?            (         )           =>    41     :    null  ;/*27*/
   false        ?            (        arg )           =>    42     :    null  ;/*28*/
    false        ?            (        arg     = 1 )           =>    43     :    null  ;/*29*/
      false        ?            (        arg    ?        )           =>    44     :    null  ;/*30*/
    false        ?            (        arg    :    number )           =>    45     :    null  ;/*31*/
   false        ?            (        arg    ?           :    number )           =>    46     :    null  ;/*32*/
      false        ?            (        arg    ?           :    number     = 0 )           =>    47     :    null  ;/*33*/
   false        ?            (                 ...     arg    :    number   [      ]    )           =>    48     :    null  ;/*34*/

// in ternary exression within paren
   false        ?            (            (         )           =>    51 )         :    null  ;/*35*/
    false        ?            (            (        arg )           =>    52 )         :    null  ;/*36*/
    false        ?            (            (        arg     = 1 )           =>    53 )         :    null  ;/*37*/
      false        ?            (            (        arg    ?        )           =>    54 )         :    null  ;/*38*/
    false        ?            (            (        arg    :    number )           =>    55 )         :    null  ;/*39*/
      false        ?            (            (        arg    ?           :    number )           =>    56 )         :    null  ;/*40*/
    false        ?            (            (        arg    ?           :    number     = 0 )           =>    57 )         :    null  ;/*41*/
   false        ?            (            (                 ...     arg    :    number   [      ]    )           =>    58 )         :    null  ;/*42*/

// ternary exression's else clause
   false        ?        null     :        (         )           =>    61  ;/*43*/
        false        ?        null     :        (        arg )           =>    62  ;/*44*/
   false        ?        null     :        (        arg     = 1 )           =>    63  ;/*45*/
      false        ?        null     :        (        arg    ?        )           =>    64  ;/*46*/
   false        ?        null     :        (        arg    :    number )           =>    65  ;/*47*/
    false        ?        null     :        (        arg    ?           :    number )           =>    66  ;/*48*/
        false        ?        null     :        (        arg    ?           :    number     = 0 )           =>    67  ;/*49*/
    false        ?        null     :        (                 ...     arg    :    number   [      ]    )           =>    68  ;/*50*/


// nested ternary expressions
    ((        a    ?        )           =>    { return a  ;    })     ?            (        b    ?         )           =>    { return b  ;    }     :        (        c    ?         )           =>    { return c  ;    }  ;/*51*/

//multiple levels
    ((        a    ?        )           =>    { return a  ;    })     ?            (        b )          =>       (        c )          =>   81     :        (        c )          =>       (        d )          =>   82  ;/*52*/


// In Expressions
    (            (        arg )           =>    90 )     instanceof Function  ;/*53*/
      (            (        arg     = 1 )           =>    91 )     instanceof Function  ;/*54*/
        (            (        arg    ?         )           =>    92 )     instanceof Function  ;/*55*/
      (            (        arg    :    number )           =>    93 )     instanceof Function  ;/*56*/
    (            (        arg    :    number     = 1 )           =>    94 )     instanceof Function  ;/*57*/
        (            (        arg    ?           :    number )           =>    95 )     instanceof Function  ;/*58*/
      (            (                 ...     arg    :    number   [      ]    )           =>    96 )     instanceof Function  ;/*59*/

''    +        ((        arg )           =>    100)  ;/*60*/
        (            (        arg )           =>    0 )        +    ''    +        ((        arg )           =>    101)  ;/*61*/
          (            (        arg     = 1 )           =>    0 )        +    ''    +        ((        arg     = 2 )           =>    102)  ;/*62*/
    (            (        arg    ?        )           =>    0 )        +    ''    +        ((        arg    ?        )           =>    103)  ;/*63*/
      (            (        arg    :   number )           =>    0 )        +    ''    +        ((        arg    :   number )           =>    104)  ;/*64*/
        (            (        arg    :   number     = 1 )           =>    0 )        +    ''    +        ((        arg    :   number     = 2 )           =>    105)  ;/*65*/
    (            (        arg    ?           :   number     )           =>    0 )        +    ''    +        ((        arg    ?           :   number     )           =>    106)  ;/*66*/
      (            (                 ...     arg    :   number   [      ]    )           =>    0 )        +    ''    +        ((                 ...     arg    :   number   [      ]    )           =>    107)  ;/*67*/
    (            (        arg1   ,    arg2    ?        )           =>    0 )        +    ''    +        ((        arg1   ,   arg2    ?        )           =>    108)  ;/*68*/
      (            (        arg1   ,             ...     arg2    :   number   [      ]    )           =>    0 )        +    ''    +        ((        arg1   ,             ...     arg2    :   number   [      ]    )           =>    108)  ;/*69*/


// Function Parameters
/*70*/function foo    (                 ...     arg    :    any   [      ]    )     { }

/*71*/foo    (
/*72*/        (        a )           =>    110   ,
/*73*/        (            (        a )           =>    111 )       ,
/*74*/        (        a )           =>    {
        return /*75*/112  ;
/*76*/    }   ,
/*77*/        (        a    ?         )           =>    113   ,
/*78*/        (        a   ,    b    ?         )           =>    114   ,
/*79*/        (        a    :    number )           =>    115   ,
/*80*/        (        a    :    number     = 0 )           =>    116   ,
/*81*/        (        a     = 0 )           =>    117   ,
/*82*/        (        a               :    number     = 0 )           =>    118   ,
/*83*/        (        a    ?    ,   b   ?          :    number      )           =>    118   ,
/*84*/        (                 ...     a    :    number   [      ]    )           =>    119   ,
/*85*/        (        a   ,    b                = 0   ,             ...     c    :    number   [      ]    )           =>    120   ,
/*86*/        (        a )           =>        (        b )           =>        (        c )           =>    121   ,
/*87*/        false       ?            (        a )           =>    0     :        (        b )           =>    122
 /*88*/)      ;"#;
    let mut s = Session::new_for_test("formattingFatArrowFunctions", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"() => 1;"#);
    fourslash::go_to_marker(&mut s, "2");
    fourslash::verify_current_line_content(&mut s, r#"(arg) => 2;"#);
    fourslash::go_to_marker(&mut s, "3");
    fourslash::verify_current_line_content(&mut s, r#"arg => 2;"#);
    fourslash::go_to_marker(&mut s, "3a");
    fourslash::verify_current_line_content(&mut s, r#"arg => 2;"#);
    fourslash::go_to_marker(&mut s, "4");
    fourslash::verify_current_line_content(&mut s, r#"(arg = 1) => 3;"#);
    fourslash::go_to_marker(&mut s, "5");
    fourslash::verify_current_line_content(&mut s, r#"(arg?) => 4;"#);
    fourslash::go_to_marker(&mut s, "6");
    fourslash::verify_current_line_content(&mut s, r#"(arg: number) => 5;"#);
    fourslash::go_to_marker(&mut s, "7");
    fourslash::verify_current_line_content(&mut s, r#"(arg: number = 0) => 6;"#);
    fourslash::go_to_marker(&mut s, "8");
    fourslash::verify_current_line_content(&mut s, r#"(arg?: number) => 7;"#);
    fourslash::go_to_marker(&mut s, "9");
    fourslash::verify_current_line_content(&mut s, r#"(...arg: number[]) => 8;"#);
    fourslash::go_to_marker(&mut s, "10");
    fourslash::verify_current_line_content(&mut s, r#"(arg1, arg2) => 12;"#);
    fourslash::go_to_marker(&mut s, "11");
    fourslash::verify_current_line_content(&mut s, r#"(arg1 = 1, arg2 = 3) => 13;"#);
    fourslash::go_to_marker(&mut s, "12");
    fourslash::verify_current_line_content(&mut s, r#"(arg1?, arg2?) => 14;"#);
    fourslash::go_to_marker(&mut s, "13");
    fourslash::verify_current_line_content(&mut s, r#"(arg1: number, arg2: number) => 15;"#);
    fourslash::go_to_marker(&mut s, "14");
    fourslash::verify_current_line_content(&mut s, r#"(arg1: number = 0, arg2: number = 1) => 16;"#);
    fourslash::go_to_marker(&mut s, "15");
    fourslash::verify_current_line_content(&mut s, r#"(arg1?: number, arg2?: number) => 17;"#);
    fourslash::go_to_marker(&mut s, "16");
    fourslash::verify_current_line_content(&mut s, r#"(arg1, ...arg2: number[]) => 18;"#);
    fourslash::go_to_marker(&mut s, "17");
    fourslash::verify_current_line_content(&mut s, r#"(arg1, arg2?: number) => 19;"#);
    fourslash::go_to_marker(&mut s, "18");
    fourslash::verify_current_line_content(&mut s, r#"(() => 21);"#);
    fourslash::go_to_marker(&mut s, "19");
    fourslash::verify_current_line_content(&mut s, r#"((arg) => 22);"#);
    fourslash::go_to_marker(&mut s, "20");
    fourslash::verify_current_line_content(&mut s, r#"((arg = 1) => 23);"#);
    fourslash::go_to_marker(&mut s, "21");
    fourslash::verify_current_line_content(&mut s, r#"((arg?) => 24);"#);
    fourslash::go_to_marker(&mut s, "22");
    fourslash::verify_current_line_content(&mut s, r#"((arg: number) => 25);"#);
    fourslash::go_to_marker(&mut s, "23");
    fourslash::verify_current_line_content(&mut s, r#"((arg: number = 0) => 26);"#);
    fourslash::go_to_marker(&mut s, "24");
    fourslash::verify_current_line_content(&mut s, r#"((arg?: number) => 27);"#);
    fourslash::go_to_marker(&mut s, "25");
    fourslash::verify_current_line_content(&mut s, r#"((...arg: number[]) => 28);"#);
    fourslash::go_to_marker(&mut s, "26");
    fourslash::verify_current_line_content(&mut s, r#"(((((arg) => { return 32; }))));"#);
    fourslash::go_to_marker(&mut s, "27");
    fourslash::verify_current_line_content(&mut s, r#"false ? () => 41 : null;"#);
    fourslash::go_to_marker(&mut s, "28");
    fourslash::verify_current_line_content(&mut s, r#"false ? (arg) => 42 : null;"#);
    fourslash::go_to_marker(&mut s, "29");
    fourslash::verify_current_line_content(&mut s, r#"false ? (arg = 1) => 43 : null;"#);
    fourslash::go_to_marker(&mut s, "30");
    fourslash::verify_current_line_content(&mut s, r#"false ? (arg?) => 44 : null;"#);
    fourslash::go_to_marker(&mut s, "31");
    fourslash::verify_current_line_content(&mut s, r#"false ? (arg: number) => 45 : null;"#);
    fourslash::go_to_marker(&mut s, "32");
    fourslash::verify_current_line_content(&mut s, r#"false ? (arg?: number) => 46 : null;"#);
    fourslash::go_to_marker(&mut s, "33");
    fourslash::verify_current_line_content(&mut s, r#"false ? (arg?: number = 0) => 47 : null;"#);
    fourslash::go_to_marker(&mut s, "34");
    fourslash::verify_current_line_content(&mut s, r#"false ? (...arg: number[]) => 48 : null;"#);
    fourslash::go_to_marker(&mut s, "35");
    fourslash::verify_current_line_content(&mut s, r#"false ? (() => 51) : null;"#);
    fourslash::go_to_marker(&mut s, "36");
    fourslash::verify_current_line_content(&mut s, r#"false ? ((arg) => 52) : null;"#);
    fourslash::go_to_marker(&mut s, "37");
    fourslash::verify_current_line_content(&mut s, r#"false ? ((arg = 1) => 53) : null;"#);
    fourslash::go_to_marker(&mut s, "38");
    fourslash::verify_current_line_content(&mut s, r#"false ? ((arg?) => 54) : null;"#);
    fourslash::go_to_marker(&mut s, "39");
    fourslash::verify_current_line_content(&mut s, r#"false ? ((arg: number) => 55) : null;"#);
    fourslash::go_to_marker(&mut s, "40");
    fourslash::verify_current_line_content(&mut s, r#"false ? ((arg?: number) => 56) : null;"#);
    fourslash::go_to_marker(&mut s, "41");
    fourslash::verify_current_line_content(&mut s, r#"false ? ((arg?: number = 0) => 57) : null;"#);
    fourslash::go_to_marker(&mut s, "42");
    fourslash::verify_current_line_content(&mut s, r#"false ? ((...arg: number[]) => 58) : null;"#);
    fourslash::go_to_marker(&mut s, "43");
    fourslash::verify_current_line_content(&mut s, r#"false ? null : () => 61;"#);
    fourslash::go_to_marker(&mut s, "44");
    fourslash::verify_current_line_content(&mut s, r#"false ? null : (arg) => 62;"#);
    fourslash::go_to_marker(&mut s, "45");
    fourslash::verify_current_line_content(&mut s, r#"false ? null : (arg = 1) => 63;"#);
    fourslash::go_to_marker(&mut s, "46");
    fourslash::verify_current_line_content(&mut s, r#"false ? null : (arg?) => 64;"#);
    fourslash::go_to_marker(&mut s, "47");
    fourslash::verify_current_line_content(&mut s, r#"false ? null : (arg: number) => 65;"#);
    fourslash::go_to_marker(&mut s, "48");
    fourslash::verify_current_line_content(&mut s, r#"false ? null : (arg?: number) => 66;"#);
    fourslash::go_to_marker(&mut s, "49");
    fourslash::verify_current_line_content(&mut s, r#"false ? null : (arg?: number = 0) => 67;"#);
    fourslash::go_to_marker(&mut s, "50");
    fourslash::verify_current_line_content(&mut s, r#"false ? null : (...arg: number[]) => 68;"#);
    fourslash::go_to_marker(&mut s, "51");
    fourslash::verify_current_line_content(&mut s, r#"((a?) => { return a; }) ? (b?) => { return b; } : (c?) => { return c; };"#);
    fourslash::go_to_marker(&mut s, "52");
    fourslash::verify_current_line_content(&mut s, r#"((a?) => { return a; }) ? (b) => (c) => 81 : (c) => (d) => 82;"#);
    fourslash::go_to_marker(&mut s, "53");
    fourslash::verify_current_line_content(&mut s, r#"((arg) => 90) instanceof Function;"#);
    fourslash::go_to_marker(&mut s, "54");
    fourslash::verify_current_line_content(&mut s, r#"((arg = 1) => 91) instanceof Function;"#);
    fourslash::go_to_marker(&mut s, "55");
    fourslash::verify_current_line_content(&mut s, r#"((arg?) => 92) instanceof Function;"#);
    fourslash::go_to_marker(&mut s, "56");
    fourslash::verify_current_line_content(&mut s, r#"((arg: number) => 93) instanceof Function;"#);
    fourslash::go_to_marker(&mut s, "57");
    fourslash::verify_current_line_content(&mut s, r#"((arg: number = 1) => 94) instanceof Function;"#);
    fourslash::go_to_marker(&mut s, "58");
    fourslash::verify_current_line_content(&mut s, r#"((arg?: number) => 95) instanceof Function;"#);
    fourslash::go_to_marker(&mut s, "59");
    fourslash::verify_current_line_content(&mut s, r#"((...arg: number[]) => 96) instanceof Function;"#);
    fourslash::go_to_marker(&mut s, "60");
    fourslash::verify_current_line_content(&mut s, r#"'' + ((arg) => 100);"#);
    fourslash::go_to_marker(&mut s, "61");
    fourslash::verify_current_line_content(&mut s, r#"((arg) => 0) + '' + ((arg) => 101);"#);
    fourslash::go_to_marker(&mut s, "62");
    fourslash::verify_current_line_content(&mut s, r#"((arg = 1) => 0) + '' + ((arg = 2) => 102);"#);
    fourslash::go_to_marker(&mut s, "63");
    fourslash::verify_current_line_content(&mut s, r#"((arg?) => 0) + '' + ((arg?) => 103);"#);
    fourslash::go_to_marker(&mut s, "64");
    fourslash::verify_current_line_content(&mut s, r#"((arg: number) => 0) + '' + ((arg: number) => 104);"#);
    fourslash::go_to_marker(&mut s, "65");
    fourslash::verify_current_line_content(&mut s, r#"((arg: number = 1) => 0) + '' + ((arg: number = 2) => 105);"#);
    fourslash::go_to_marker(&mut s, "66");
    fourslash::verify_current_line_content(&mut s, r#"((arg?: number) => 0) + '' + ((arg?: number) => 106);"#);
    fourslash::go_to_marker(&mut s, "67");
    fourslash::verify_current_line_content(&mut s, r#"((...arg: number[]) => 0) + '' + ((...arg: number[]) => 107);"#);
    fourslash::go_to_marker(&mut s, "68");
    fourslash::verify_current_line_content(&mut s, r#"((arg1, arg2?) => 0) + '' + ((arg1, arg2?) => 108);"#);
    fourslash::go_to_marker(&mut s, "69");
    fourslash::verify_current_line_content(&mut s, r#"((arg1, ...arg2: number[]) => 0) + '' + ((arg1, ...arg2: number[]) => 108);"#);
    fourslash::go_to_marker(&mut s, "70");
    fourslash::verify_current_line_content(&mut s, r#"function foo(...arg: any[]) { }"#);
    fourslash::go_to_marker(&mut s, "71");
    fourslash::verify_current_line_content(&mut s, r#"foo("#);
    fourslash::go_to_marker(&mut s, "72");
    fourslash::verify_current_line_content(&mut s, r#"    (a) => 110,"#);
    fourslash::go_to_marker(&mut s, "73");
    fourslash::verify_current_line_content(&mut s, r#"    ((a) => 111),"#);
    fourslash::go_to_marker(&mut s, "74");
    fourslash::verify_current_line_content(&mut s, r#"    (a) => {"#);
    fourslash::go_to_marker(&mut s, "75");
    fourslash::verify_current_line_content(&mut s, r#"        return 112;"#);
    fourslash::go_to_marker(&mut s, "76");
    fourslash::verify_current_line_content(&mut s, r#"    },"#);
    fourslash::go_to_marker(&mut s, "77");
    fourslash::verify_current_line_content(&mut s, r#"    (a?) => 113,"#);
    fourslash::go_to_marker(&mut s, "78");
    fourslash::verify_current_line_content(&mut s, r#"    (a, b?) => 114,"#);
    fourslash::go_to_marker(&mut s, "79");
    fourslash::verify_current_line_content(&mut s, r#"    (a: number) => 115,"#);
    fourslash::go_to_marker(&mut s, "80");
    fourslash::verify_current_line_content(&mut s, r#"    (a: number = 0) => 116,"#);
    fourslash::go_to_marker(&mut s, "81");
    fourslash::verify_current_line_content(&mut s, r#"    (a = 0) => 117,"#);
    fourslash::go_to_marker(&mut s, "82");
    fourslash::verify_current_line_content(&mut s, r#"    (a: number = 0) => 118,"#);
    fourslash::go_to_marker(&mut s, "83");
    fourslash::verify_current_line_content(&mut s, r#"    (a?, b?: number) => 118,"#);
    fourslash::go_to_marker(&mut s, "84");
    fourslash::verify_current_line_content(&mut s, r#"    (...a: number[]) => 119,"#);
    fourslash::go_to_marker(&mut s, "85");
    fourslash::verify_current_line_content(&mut s, r#"    (a, b = 0, ...c: number[]) => 120,"#);
    fourslash::go_to_marker(&mut s, "86");
    fourslash::verify_current_line_content(&mut s, r#"    (a) => (b) => (c) => 121,"#);
    fourslash::go_to_marker(&mut s, "87");
    fourslash::verify_current_line_content(&mut s, r#"    false ? (a) => 0 : (b) => 122"#);
}
