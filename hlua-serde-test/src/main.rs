extern crate hlua;
extern crate serde_json;
extern crate serde_test;

use serde_test::{assert_tokens, Token};

use hlua::AnyLuaValue::*;
use hlua::{AnyLuaString, AnyLuaValue};

fn main() {
    let v1 = LuaString(String::from("abc"));
    let v2 = LuaAnyString(AnyLuaString(vec![b'a', b'b', b'c']));
    let v3 = LuaNumber(42.);
    let v4 = LuaBoolean(true);
    let v5 = LuaArray(vec![
        (LuaNumber(42.), LuaString(String::from("foo"))),
        (LuaBoolean(true), LuaString(String::from("foo"))),
    ]);
    let v6 = LuaNil;
    let v7 = LuaArray(vec![
        (LuaString(String::from("foo")), LuaNumber(42.)),
        (LuaString(String::from("foo")), LuaBoolean(true)),
    ]);
    let v8 = LuaArray(vec![
        (
            LuaString(String::from("test_result")),
            LuaString(String::from("pass")),
        ),
        (
            LuaString(String::from("results")),
            LuaArray(vec![
                (LuaNumber(1.0), LuaBoolean(true)),
                (
                    LuaNumber(2.0),
                    LuaArray(vec![
                        (LuaString(String::from("time")), LuaNumber(367000000.)),
                        (LuaString(String::from("size")), LuaNumber(1000.)),
                        (LuaString(String::from("number")), LuaNumber(1000.)),
                        (LuaString(String::from("rate")), LuaNumber(0.)),
                        (LuaString(String::from("res_sec")), LuaNumber(0.)),
                        (LuaString(String::from("res_nsec")), LuaNumber(1000000.)),
                    ]),
                ),
            ]),
        ),
    ]);

    assert_tokens(&v1, &[Token::String("abc")]);

    assert_tokens(
        &v2,
        &[
            Token::NewtypeStruct {
                name: "AnyLuaString",
            },
            Token::Seq { len: Some(3) },
            Token::U8(97),
            Token::U8(98),
            Token::U8(99),
            Token::SeqEnd,
        ],
    );

    assert_tokens(&v3, &[Token::F64(42.)]);

    assert_tokens(&v4, &[Token::Bool(true)]);

    assert_tokens(
        &v5,
        &[
            Token::Map { len: Some(2) },
            Token::String("42"),
            Token::String("foo"),
            Token::String("true"),
            Token::String("foo"),
            Token::MapEnd,
        ],
    );

    assert_tokens(&v6, &[Token::Unit]);

    assert_tokens(
        &v7,
        &[
            Token::Map { len: Some(2) },
            Token::String("foo"),
            Token::F64(42.),
            Token::String("foo"),
            Token::Bool(true),
            Token::MapEnd,
        ],
    );

    assert_tokens(
        &v8,
        &[
            Token::Map { len: Some(2) },
            Token::String("test_result"),
            Token::String("pass"),
            Token::String("results"),
            Token::Map { len: Some(2) },
            Token::String("1"),
            Token::Bool(true),
            Token::String("2"),
            Token::Map { len: Some(6) },
            Token::String("time"),
            Token::F64(367000000.),
            Token::String("size"),
            Token::F64(1000.),
            Token::String("number"),
            Token::F64(1000.),
            Token::String("rate"),
            Token::F64(0.),
            Token::String("res_sec"),
            Token::F64(0.),
            Token::String("res_nsec"),
            Token::F64(1000000.),
            Token::MapEnd,
            Token::MapEnd,
            Token::MapEnd,
        ],
    );

    let values = [v1, v2, v3, v4, v5, v6, v7, v8];
    for v in values.iter() {
        let s = serde_json::to_string(&v).unwrap();
        println!("{}", s);
        let v_: AnyLuaValue = serde_json::from_str(&s).unwrap();
        assert_eq!(*v, v_);
    }
}
