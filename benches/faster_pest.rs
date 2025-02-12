#![feature(test)]

extern crate test;

use test::Bencher;
use faster_pest::*;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "src/json.pest"]
pub struct JsonParser;

#[derive(Debug)]
enum Value<'i> {
    String(Cow<'i, str>),
    Number(f64),
    Boolean(bool),
    Array(Vec<Value<'i>>),
    Object(HashMap<Cow<'i, str>, Value<'i>>),
    Null,
}

impl<'i> Value<'i> {
    fn from_ident_ref(value: IdentRef<'i, Ident>) -> Self {
        match value.as_rule() {
            Rule::string => Value::String(unescape(value)),
            Rule::number => Value::Number(value.as_str().parse().unwrap()),
            Rule::boolean => Value::Boolean(value.as_str() == "true"),
            Rule::array => {
                let mut array = Vec::new();
                array.extend(value.children().map(Value::from_ident_ref));
                Value::Array(array)
            }
            Rule::object => {
                let mut object = HashMap::new();
                for property in value.children() {
                    let mut property_children = property.children();
                    let name = property_children.next().unwrap();
                    let name = unescape(name);
                    let value = property_children.next().unwrap();
                    object.insert(name, Value::from_ident_ref(value));
                }
                Value::Object(object)
            }
            Rule::null => Value::Null,
            Rule::property | Rule::file | Rule::escaped_char => unreachable!(),
        }
    }
}

fn unescape<'i>(s: IdentRef<'i, Ident>) -> Cow<'i, str> {
    let children_count = s.children_count();
    if children_count == 0 {
        return Cow::Borrowed(s.as_str());
    }
    let mut unescaped = String::with_capacity(s.as_str().len() - children_count);
    let mut i = 0;
    let start_addr = s.as_str().as_ptr() as usize;
    for escaped_char in s.children() {
        let end = escaped_char.as_str().as_ptr() as usize - start_addr;
        unescaped.push_str(unsafe { s.as_str().get_unchecked(i..end) });
        match unsafe { escaped_char.as_str().as_bytes().get_unchecked(1) } {
            b'"' => unescaped.push('"'),
            b'\\' => unescaped.push('\\'),
            b'/' => unescaped.push('/'),
            b'b' => unescaped.push('\x08'),
            b'f' => unescaped.push('\x0c'),
            b'n' => unescaped.push('\n'),
            b'r' => unescaped.push('\r'),
            b't' => unescaped.push('\t'),
            b'u' => {
                // Warning when you implement this, you might want to increase the capacity of the string set above
                unimplemented!()
            }
            _ => unreachable!()
        }
        i = end + 2;
    }
    unescaped.push_str(unsafe { s.as_str().get_unchecked(i..) });
    Cow::Owned(unescaped)
}

const CANADA : &str = include_str!("../assets/canada.json");
const DATA   : &str = include_str!("../assets/data.json");

#[bench]
fn faster_pest_canada_shallow(b: &mut Bencher) {
    b.iter(|| {
        JsonParser::parse_file(CANADA).expect("unsuccessful parse");
    });
}

#[bench]
fn faster_pest_canada(b: &mut Bencher) {
    b.iter(|| {
        let output = JsonParser::parse_file(CANADA).map_err(|e| e.print(CANADA)).unwrap();
        let file = output.into_iter().next().unwrap();
        let main_object = file.children().next().unwrap();
        let output = Value::from_ident_ref(main_object);
    });
}

#[bench]
fn faster_pest_data_shallow(b: &mut Bencher) {
    b.iter(|| {
        JsonParser::parse_file(DATA).expect("unsuccessful parse");
    });
}

#[bench]
fn faster_pest_data(b: &mut Bencher) {
    b.iter(|| {
        let output = JsonParser::parse_file(DATA).map_err(|e| e.print(DATA)).unwrap();
        let file = output.into_iter().next().unwrap();
        let main_object = file.children().next().unwrap();
        let output = Value::from_ident_ref(main_object);
    });
}
