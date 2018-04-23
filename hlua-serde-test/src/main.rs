extern crate hlua;
extern crate serde_json;
extern crate serde_test;

use serde_test::{assert_tokens, Token};

use hlua::{AnyLuaString, AnyLuaValue};

fn main() {
    let v1 = AnyLuaValue::LuaString(String::from("abc"));
    let v2 = AnyLuaValue::LuaAnyString(AnyLuaString(vec![b'a', b'b', b'c']));
    let v3 = AnyLuaValue::LuaNumber(42.);
    let v4 = AnyLuaValue::LuaBoolean(true);
    let v5 = AnyLuaValue::LuaArray(vec![
        (
            AnyLuaValue::LuaNumber(42.),
            AnyLuaValue::LuaString(String::from("foo")),
        ),
        (
            AnyLuaValue::LuaBoolean(true),
            AnyLuaValue::LuaString(String::from("foo")),
        ),
    ]);
    let v6 = AnyLuaValue::LuaNil;
    let v7 = AnyLuaValue::LuaArray(vec![
        (
            AnyLuaValue::LuaString(String::from("foo")),
            AnyLuaValue::LuaNumber(42.),
        ),
        (
            AnyLuaValue::LuaString(String::from("foo")),
            AnyLuaValue::LuaBoolean(true),
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

    let values = [v1, v2, v3, v4, v5, v6, v7];
    for v in values.iter() {
        let s = serde_json::to_string(&v).unwrap();
        println!("{}", s);
        let v_: AnyLuaValue = serde_json::from_str(&s).unwrap();
        assert_eq!(*v, v_);
    }
}
