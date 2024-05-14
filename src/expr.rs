// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use serde_json::Value;
use std::cmp::Ordering;
use std::iter;

use crate::workflow::Comparison;

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
    #[allow(clippy::match_same_arms)]
    match (comparison, partial_cmp_json_values(a, b)) {
        (Comparison::LessThan, Some(Ordering::Less)) => Some(true),
        (Comparison::LessThanOrEqualTo, Some(Ordering::Less | Ordering::Equal)) => Some(true),
        (Comparison::EqualTo, Some(Ordering::Equal)) => Some(true),
        (Comparison::GreaterThanOrEqualTo, Some(Ordering::Greater | Ordering::Equal)) => Some(true),
        (Comparison::GreaterThan, Some(Ordering::Greater)) => Some(true),
        (_, None) => None,
        (_, _) => Some(false),
    }
}

#[cfg(test)]
mod tests {
    use serial_test::parallel;

    use super::*;

    #[test]
    #[parallel]
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
    #[parallel]
    fn cmp_invalid_types_json() {
        assert_eq!(
            partial_cmp_json_values(&Value::from(10), &Value::from("abcd")),
            None
        );
    }
    #[test]
    #[parallel]
    fn cmp_invalid_tuple_json() {
        let a = Value::Array(vec![Value::from(14), Value::from("j"), Value::from(3.5)]);
        let b = Value::Array(vec![Value::from(13), Value::from("j")]);
        assert_eq!(partial_cmp_json_values(&a, &b), None);
    }

    #[test]
    #[parallel]
    fn eval() {
        assert_eq!(
            evaluate_json_comparison(&Comparison::EqualTo, &Value::from(5), &Value::from(5)),
            Some(true)
        );
        assert_eq!(
            evaluate_json_comparison(
                &Comparison::GreaterThanOrEqualTo,
                &Value::from(5),
                &Value::from(5)
            ),
            Some(true)
        );
        assert_eq!(
            evaluate_json_comparison(
                &Comparison::LessThanOrEqualTo,
                &Value::from(5),
                &Value::from(5)
            ),
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
            evaluate_json_comparison(
                &Comparison::GreaterThanOrEqualTo,
                &Value::from(5),
                &Value::from(10)
            ),
            Some(false)
        );
        assert_eq!(
            evaluate_json_comparison(
                &Comparison::GreaterThanOrEqualTo,
                &Value::from(6),
                &Value::from(5)
            ),
            Some(true)
        );
        assert_eq!(
            evaluate_json_comparison(&Comparison::LessThan, &Value::from(5), &Value::from(10)),
            Some(true)
        );
        assert_eq!(
            evaluate_json_comparison(
                &Comparison::LessThanOrEqualTo,
                &Value::from(5),
                &Value::from(10)
            ),
            Some(true)
        );
        assert_eq!(
            evaluate_json_comparison(
                &Comparison::LessThanOrEqualTo,
                &Value::from(5),
                &Value::from(4)
            ),
            Some(false)
        );
    }
}
