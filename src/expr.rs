use serde_json::Value;
use std::cmp::Ordering;
use std::iter;

use crate::workflow::Comparison;

// Commented code related to evalexpr. Will test with users first and see if they find a need for
// more flexible expressions and whether it is worth the complexity. Also evalexpr appears to be
// no longer actively maintained.

// use evalexpr::{ContextWithMutableVariables, HashMapContext, TupleType};

// /// Convert a JSON Tuple element to the evalexpr Value.
// ///
// /// evalexpr has no way to store maps as tuple elements. Store them as JSON
// /// strings to preserve some information.
// ///
// fn value_to_tuple_element(value: &Value) -> Result<evalexpr::Value, Error> {
//     Ok(match value {
//         Value::Object(_obj) => {
//             todo!("Implement maps in arrays");
//         }
//         Value::String(string) => evalexpr::Value::String(string.clone()),
//         Value::Array(array) => {
//             let tuple: TupleType = array
//                 .iter()
//                 .map(value_to_tuple_element)
//                 .collect::<Result<_, _>>()?;
//             evalexpr::Value::Tuple(tuple)
//         }
//         Value::Bool(bool) => evalexpr::Value::Boolean(*bool),
//         Value::Null => evalexpr::Value::Empty,
//         Value::Number(number) => {
//             if let Some(v) = number.as_i64() {
//                 evalexpr::Value::Int(v)
//             } else if let Some(v) = number.as_f64() {
//                 evalexpr::Value::Float(v)
//             } else {
//                 return Err(Error::InvalidNumber(number.to_string()));
//             }
//         }
//     })
// }

// /// Recursively build up a context
// ///
// /// Inspired by https://github.com/ISibboI/evalexpr/issues/117#issuecomment-1792496021
// ///
// fn add_value_to_context(
//     prefix: &str,
//     value: &Value,
//     context: &mut HashMapContext,
// ) -> Result<(), Error> {
//     match value {
//         Value::Object(obj) => {
//             for (key, value) in obj {
//                 let new_key = if prefix.is_empty() {
//                     key.to_string()
//                 } else {
//                     format!("{}.{}", prefix, key)
//                 };
//                 add_value_to_context(&new_key, value, context)?;
//             }
//         }
//         Value::String(string) => {
//             context.set_value(prefix.into(), evalexpr::Value::String(string.clone()))?;
//         }
//         Value::Array(array) => {
//             let tuple: TupleType = array
//                 .iter()
//                 .map(value_to_tuple_element)
//                 .collect::<Result<_, _>>()?;
//             context.set_value(prefix.into(), evalexpr::Value::Tuple(tuple))?;
//         }
//         Value::Bool(bool) => {
//             context.set_value(prefix.into(), evalexpr::Value::Boolean(*bool))?;
//         }
//         Value::Null => {
//             context.set_value(prefix.into(), evalexpr::Value::Empty)?;
//         }
//         Value::Number(number) => {
//             if let Some(v) = number.as_i64() {
//                 context.set_value(prefix.into(), evalexpr::Value::Int(v))?;
//             } else if let Some(v) = number.as_f64() {
//                 context.set_value(prefix.into(), evalexpr::Value::Float(v))?;
//             } else {
//                 return Err(Error::InvalidNumber(number.to_string()));
//             }
//         }
//     }

//     Ok(())
// }

// /// Convert a JSON value to an evalexpr::HashMapContext.
// ///
// /// The top level map is flattened with its keys becoming variables. Nested
// /// maps are expanded with a dotted notation.
// ///
// fn json_value_to_context(base: &str, value: &Value) -> Result<HashMapContext, Error> {
//     let mut context = HashMapContext::new();
//     add_value_to_context(base, value, &mut context)?;
//     Ok(context)
// }

// /// Compare two expreval::Values lexicographically.
// ///
// /// # Panics
// /// Panics when the two values are not the same type.
// ///
// fn cmp_values(a: &evalexpr::Value, b: &evalexpr::Value) -> Ordering {
//     match a {
//         evalexpr::Value::String(a_str) => {
//             if let evalexpr::Value::String(b_str) = b {
//                 a_str.cmp(b_str)
//             } else {
//                 panic!("Cannot compare {:?} and {:?}", a, b);
//             }
//         }
//         evalexpr::Value::Float(a_float) => {
//             if let evalexpr::Value::Float(b_float) = b {
//                 a_float.partial_cmp(b_float).expect(&format!(
//                     "Valid floating point values, got: {:?} and {:?}",
//                     a, b
//                 ))
//             } else {
//                 panic!("Cannot compare {:?} and {:?}", a, b);
//             }
//         }
//         evalexpr::Value::Int(a_int) => {
//             if let evalexpr::Value::Int(b_int) = b {
//                 a_int.cmp(b_int)
//             } else {
//                 panic!("Cannot compare {:?} and {:?}", a, b);
//             }
//         }
//         evalexpr::Value::Boolean(a_bool) => {
//             if let evalexpr::Value::Boolean(b_bool) = b {
//                 a_bool.cmp(b_bool)
//             } else {
//                 panic!("Cannot compare {:?} and {:?}", a, b);
//             }
//         }
//         evalexpr::Value::Empty => {
//             if b.is_empty() {
//                 Ordering::Equal
//             } else {
//                 panic!("Cannot compare {:?} and {:?}", a, b);
//             }
//         }
//         evalexpr::Value::Tuple(a_tuple) => {
//             if let evalexpr::Value::Tuple(b_tuple) = b {
//                 if a_tuple.len() != b_tuple.len() {
//                     panic!("Cannot compare {:?} and {:?}", a, b);
//                 }

//                 if a_tuple.is_empty() && b_tuple.is_empty() {
//                     Ordering::Equal
//                 } else {
//                     for (c, d) in iter::zip(a_tuple, b_tuple) {
//                         match cmp_values(c, d) {
//                             Ordering::Less => return Ordering::Less,
//                             Ordering::Greater => return Ordering::Greater,
//                             Ordering::Equal => (),
//                         };
//                     }
//                     Ordering::Equal
//                 }
//             } else {
//                 panic!("Cannot compare {:?} and {:?}", a, b);
//             }
//         }
//     }
// }

/// Compares two Values lexicographically.
///
/// # Returns
/// `Some(Ordering)` when an ordering can be determined, otherwise `None`.
///
pub(crate) fn partial_cmp_json_values(a: &Value, b: &Value) -> Option<Ordering> {
    match (a, b) {
        (Value::String(a_str), Value::String(b_str)) => Some(a_str.cmp(b_str)),
        (Value::Bool(a_bool), Value::Bool(b_bool)) => Some(a_bool.cmp(b_bool)),
        (Value::Null, Value::Null) => Some(Ordering::Equal),
        (Value::Number(a_number), Value::Number(b_number)) => {
            match (a_number.as_i64(), b_number.as_i64()) {
                (Some(a_int), Some(b_int)) => Some(a_int.cmp(&b_int)),
                (_, _) => match (a_number.as_f64(), b_number.as_f64()) {
                    (Some(a_float), Some(b_float)) => a_float.partial_cmp(&b_float),
                    (_, _) => None,
                },
            }
        }
        (Value::Array(a_array), Value::Array(b_array)) => {
            if a_array.len() != b_array.len() {
                return None;
            }

            if a_array.is_empty() && b_array.is_empty() {
                Some(Ordering::Equal)
            } else {
                for (c, d) in iter::zip(a_array, b_array) {
                    match partial_cmp_json_values(c, d) {
                        Some(Ordering::Less) => return Some(Ordering::Less),
                        Some(Ordering::Greater) => return Some(Ordering::Greater),
                        None => return None,
                        Some(Ordering::Equal) => (),
                    };
                }
                Some(Ordering::Equal)
            }
        }
        (_, _) => None,
    }
}

/// Compares two Values lexicographically with the given comparison operator.
///
/// # Returns
/// `Some(Ordering)` when an ordering can be determined, otherwise `None`.
///
pub(crate) fn evaluate_json_comparison(
    comparison: &Comparison,
    a: &Value,
    b: &Value,
) -> Option<bool> {
    match (comparison, partial_cmp_json_values(a, b)) {
        (Comparison::EqualTo, Some(Ordering::Equal)) => Some(true),
        (Comparison::GreaterThan, Some(Ordering::Greater)) => Some(true),
        (Comparison::LessThan, Some(Ordering::Less)) => Some(true),
        (_, None) => None,
        (_, _) => Some(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // use evalexpr::{Context, IterateVariablesContext, Value};
    // use serde_json::json;

    // #[test]
    // fn string() -> Result<(), Box<dyn std::error::Error>> {
    //     let str = "this is a string";
    //     let json_value = json!(str);
    //     let result = json_value_to_context("v", &json_value)?;

    //     assert_eq!(result.iter_variables().count(), 1);
    //     assert_eq!(result.get_value("v"), Some(&Value::String(str.into())));
    //     Ok(())
    // }

    // #[test]
    // fn bool() -> Result<(), Box<dyn std::error::Error>> {
    //     let bool = true;
    //     let json_value = json!(bool);
    //     let result = json_value_to_context("v", &json_value)?;

    //     assert_eq!(result.iter_variables().count(), 1);
    //     assert_eq!(result.get_value("v"), Some(&Value::Boolean(bool)));
    //     Ok(())
    // }

    // #[test]
    // fn empty() -> Result<(), Box<dyn std::error::Error>> {
    //     let json_value = Value::Null;
    //     let result = json_value_to_context("v", &json_value)?;

    //     assert_eq!(result.iter_variables().count(), 1);
    //     assert_eq!(result.get_value("v"), Some(&Value::Empty));
    //     Ok(())
    // }

    // #[test]
    // fn integer() -> Result<(), Box<dyn std::error::Error>> {
    //     let int = 1_321_987_654;
    //     let json_value = json!(int);
    //     let result = json_value_to_context("v", &json_value)?;

    //     assert_eq!(result.iter_variables().count(), 1);
    //     assert_eq!(result.get_value("v"), Some(&Value::Int(int)));
    //     Ok(())
    // }

    // #[test]
    // fn float() -> Result<(), Box<dyn std::error::Error>> {
    //     let float = 1.234e16;
    //     let json_value = json!(float);
    //     let result = json_value_to_context("v", &json_value)?;

    //     assert_eq!(result.iter_variables().count(), 1);
    //     assert_eq!(result.get_value("v"), Some(&Value::Float(float)));
    //     Ok(())
    // }

    // #[test]
    // fn array() -> Result<(), Box<dyn std::error::Error>> {
    //     let json_value = json!(["a", 1, 3.5, true]);
    //     let result = json_value_to_context("v", &json_value)?;

    //     assert_eq!(result.iter_variables().count(), 1);
    //     let expected = vec![
    //         Value::String("a".into()),
    //         Value::Int(1),
    //         Value::Float(3.5),
    //         Value::Boolean(true),
    //     ];
    //     assert_eq!(result.get_value("v"), Some(&Value::Tuple(expected)));
    //     Ok(())
    // }

    // #[test]
    // fn array_array() -> Result<(), Box<dyn std::error::Error>> {
    //     let json_value = json!([[1, 2], [3, 4, 5]]);
    //     let result = json_value_to_context("v", &json_value)?;

    //     assert_eq!(result.iter_variables().count(), 1);
    //     let zero = vec![Value::Int(1), Value::Int(2)];
    //     let one = vec![Value::Int(3), Value::Int(4), Value::Int(5)];
    //     let expected = vec![Value::Tuple(zero), Value::Tuple(one)];
    //     assert_eq!(result.get_value("v"), Some(&Value::Tuple(expected)));
    //     Ok(())
    // }

    // #[test]
    // fn map() -> Result<(), Box<dyn std::error::Error>> {
    //     let json_value = json!({
    //         "a": "b",
    //         "c": 10,
    //         "d": -12.5,
    //         "e": {"f": 1, "g": "h"},
    //         "i": [14, "j", 3.5],
    //         "l": null
    //     });
    //     let result = json_value_to_context("v", &json_value)?;

    //     assert_eq!(result.iter_variables().count(), 7);
    //     assert_eq!(result.get_value("v.a"), Some(&Value::from("b")));
    //     assert_eq!(result.get_value("v.c"), Some(&Value::Int(10)));
    //     assert_eq!(result.get_value("v.d"), Some(&Value::Float(-12.5)));
    //     assert_eq!(result.get_value("v.e.f"), Some(&Value::Int(1)));
    //     assert_eq!(result.get_value("v.e.g"), Some(&Value::from("h")));
    //     assert_eq!(result.get_value("v.e.g"), Some(&Value::from("h")));
    //     let expected = vec![Value::Int(14), Value::from("j"), Value::Float(3.5)];
    //     assert_eq!(result.get_value("v.i"), Some(&Value::Tuple(expected)));
    //     assert_eq!(result.get_value("v.l"), Some(&Value::Empty));
    //     Ok(())
    // }

    // #[test]
    // fn cmp_valid() {
    //     assert_eq!(cmp_values(&Value::Int(0), &Value::Int(10)), Ordering::Less);
    //     assert_eq!(
    //         cmp_values(&Value::Int(10), &Value::Int(0)),
    //         Ordering::Greater
    //     );
    //     assert_eq!(cmp_values(&Value::Int(0), &Value::Int(0)), Ordering::Equal);
    //     assert_eq!(
    //         cmp_values(&Value::from("abcd"), &Value::from("abce")),
    //         Ordering::Less
    //     );
    //     assert_eq!(
    //         cmp_values(&Value::from("abcd"), &Value::from("abcd")),
    //         Ordering::Equal
    //     );
    //     assert_eq!(
    //         cmp_values(&Value::from("abce"), &Value::from("abcd")),
    //         Ordering::Greater
    //     );
    //     assert_eq!(
    //         cmp_values(&Value::from(1.0), &Value::from(2.0)),
    //         Ordering::Less
    //     );
    //     assert_eq!(
    //         cmp_values(&Value::from(1.0), &Value::from(1.0)),
    //         Ordering::Equal
    //     );
    //     assert_eq!(
    //         cmp_values(&Value::from(2.0), &Value::from(1.0)),
    //         Ordering::Greater
    //     );
    //     assert_eq!(
    //         cmp_values(&Value::from(false), &Value::from(true)),
    //         Ordering::Less
    //     );
    //     assert_eq!(
    //         cmp_values(&Value::from(true), &Value::from(false)),
    //         Ordering::Greater
    //     );
    //     assert_eq!(
    //         cmp_values(&Value::from(true), &Value::from(true)),
    //         Ordering::Equal
    //     );
    //     assert_eq!(cmp_values(&Value::Empty, &Value::Empty), Ordering::Equal);

    //     let a = Value::Tuple(vec![Value::Int(14), Value::from("j"), Value::Float(3.5)]);
    //     let b = Value::Tuple(vec![Value::Int(13), Value::from("j"), Value::Float(3.5)]);
    //     let c = Value::Tuple(vec![Value::Int(13), Value::from("j"), Value::Float(3.0)]);

    //     assert_eq!(cmp_values(&a, &b), Ordering::Greater);
    //     assert_eq!(cmp_values(&b, &a), Ordering::Less);
    //     assert_eq!(cmp_values(&b, &c), Ordering::Greater);
    // }

    // #[test]
    // #[should_panic]
    // fn cmp_invalid_float_int() {
    //     cmp_values(&Value::from(10), &Value::from(3.5));
    // }
    // #[test]
    // #[should_panic]
    // fn cmp_invalid_tuple() {
    //     let a = Value::Tuple(vec![Value::Int(14), Value::from("j"), Value::Float(3.5)]);
    //     let b = Value::Tuple(vec![Value::Int(13), Value::from("j")]);
    //     cmp_values(&a, &b);
    // }

    #[test]
    fn cmp_valid_json() {
        assert_eq!(
            partial_cmp_json_values(&Value::from(0), &Value::from(10)),
            Some(Ordering::Less)
        );
        assert_eq!(
            partial_cmp_json_values(&Value::from(10), &Value::from(0)),
            Some(Ordering::Greater)
        );
        assert_eq!(
            partial_cmp_json_values(&Value::from(0), &Value::from(0)),
            Some(Ordering::Equal)
        );
        assert_eq!(
            partial_cmp_json_values(&Value::from("abcd"), &Value::from("abce")),
            Some(Ordering::Less)
        );
        assert_eq!(
            partial_cmp_json_values(&Value::from("abcd"), &Value::from("abcd")),
            Some(Ordering::Equal)
        );
        assert_eq!(
            partial_cmp_json_values(&Value::from("abce"), &Value::from("abcd")),
            Some(Ordering::Greater)
        );
        assert_eq!(
            partial_cmp_json_values(&Value::from(1.0), &Value::from(2.0)),
            Some(Ordering::Less)
        );
        assert_eq!(
            partial_cmp_json_values(&Value::from(1.0), &Value::from(1.0)),
            Some(Ordering::Equal)
        );
        assert_eq!(
            partial_cmp_json_values(&Value::from(2.0), &Value::from(1.0)),
            Some(Ordering::Greater)
        );
        assert_eq!(
            partial_cmp_json_values(&Value::from(false), &Value::from(true)),
            Some(Ordering::Less)
        );
        assert_eq!(
            partial_cmp_json_values(&Value::from(true), &Value::from(false)),
            Some(Ordering::Greater)
        );
        assert_eq!(
            partial_cmp_json_values(&Value::from(true), &Value::from(true)),
            Some(Ordering::Equal)
        );
        assert_eq!(
            partial_cmp_json_values(&Value::Null, &Value::Null),
            Some(Ordering::Equal)
        );

        let a = Value::Array(vec![Value::from(14), Value::from("j"), Value::from(3.5)]);
        let b = Value::Array(vec![Value::from(13), Value::from("j"), Value::from(3.5)]);
        let c = Value::Array(vec![Value::from(13), Value::from("j"), Value::from(3.0)]);

        assert_eq!(partial_cmp_json_values(&a, &b), Some(Ordering::Greater));
        assert_eq!(partial_cmp_json_values(&b, &a), Some(Ordering::Less));
        assert_eq!(partial_cmp_json_values(&b, &c), Some(Ordering::Greater));
    }

    #[test]
    fn cmp_invalid_types_json() {
        assert_eq!(
            partial_cmp_json_values(&Value::from(10), &Value::from("abcd")),
            None
        );
    }
    #[test]
    fn cmp_invalid_tuple_json() {
        let a = Value::Array(vec![Value::from(14), Value::from("j"), Value::from(3.5)]);
        let b = Value::Array(vec![Value::from(13), Value::from("j")]);
        assert_eq!(partial_cmp_json_values(&a, &b), None);
    }

    #[test]
    fn eval() {
        assert_eq!(
            evaluate_json_comparison(&Comparison::EqualTo, &Value::from(5), &Value::from(5)),
            Some(true)
        );
        assert_eq!(
            evaluate_json_comparison(&Comparison::EqualTo, &Value::from(5), &Value::from(10)),
            Some(false)
        );
        assert_eq!(
            evaluate_json_comparison(&Comparison::GreaterThan, &Value::from(5), &Value::from(10)),
            Some(false)
        );
        assert_eq!(
            evaluate_json_comparison(&Comparison::LessThan, &Value::from(5), &Value::from(10)),
            Some(true)
        );
    }
}
