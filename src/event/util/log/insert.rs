use super::{Atom, PathComponent, PathIter, Value};
use std::{collections::BTreeMap, iter::Peekable};

/// Inserts field value using a path specified using `a.b[1].c` notation.
pub fn insert(fields: &mut BTreeMap<Atom, Value>, path: &str, value: Value) {
    map_insert(fields, PathIter::new(path).peekable(), value);
}

fn map_insert<I>(fields: &mut BTreeMap<Atom, Value>, mut path_iter: Peekable<I>, value: Value)
where
    I: Iterator<Item = PathComponent>,
{
    match (path_iter.next(), path_iter.peek()) {
        (Some(PathComponent::Key(current)), None) => {
            fields.insert(current, value);
        }
        (Some(PathComponent::Key(current)), Some(PathComponent::Key(_))) => {
            if let Some(Value::Map(map)) = fields.get_mut(&current) {
                map_insert(map, path_iter, value);
            } else {
                let mut map = BTreeMap::new();
                map_insert(&mut map, path_iter, value);
                fields.insert(current, Value::Map(map));
            }
        }
        (Some(PathComponent::Key(current)), Some(&PathComponent::Index(next))) => {
            if let Some(Value::Array(array)) = fields.get_mut(&current) {
                array_insert(array, path_iter, value);
            } else {
                let mut array = Vec::with_capacity(next + 1);
                array_insert(&mut array, path_iter, value);
                fields.insert(current, Value::Array(array));
            }
        }
        _ => return,
    }
}

fn array_insert<I>(values: &mut Vec<Value>, mut path_iter: Peekable<I>, value: Value)
where
    I: Iterator<Item = PathComponent>,
{
    match (path_iter.next(), path_iter.peek()) {
        (Some(PathComponent::Index(current)), None) => {
            while values.len() < current {
                values.push(Value::Null);
            }
            values.insert(current, value);
        }
        (Some(PathComponent::Index(current)), Some(PathComponent::Key(_))) => {
            if let Some(Value::Map(map)) = values.get_mut(current) {
                map_insert(map, path_iter, value);
            } else {
                let mut map = BTreeMap::new();
                map_insert(&mut map, path_iter, value);
                while values.len() < current {
                    values.push(Value::Null);
                }
                values.insert(current, Value::Map(map));
            }
        }
        (Some(PathComponent::Index(current)), Some(PathComponent::Index(next))) => {
            if let Some(Value::Array(array)) = values.get_mut(current) {
                array_insert(array, path_iter, value);
            } else {
                let mut array = Vec::with_capacity(next + 1);
                array_insert(&mut array, path_iter, value);
                while values.len() < current {
                    values.push(Value::Null);
                }
                values.insert(current, Value::Array(array));
            }
        }
        _ => return,
    }
}

#[cfg(test)]
mod test {
    use super::super::test::fields_from_json;
    use super::*;
    use serde_json::json;
    use std::collections::BTreeMap;

    #[test]
    fn test_insert_nested() {
        let mut fields = BTreeMap::new();
        insert(&mut fields, "a.b.c".into(), Value::Integer(3));

        let expected = fields_from_json(json!({
            "a": {
                "b":{
                    "c": 3
                }
            }
        }));
        assert_eq!(fields, expected);
    }

    #[test]
    fn test_insert_array() {
        let mut fields = BTreeMap::new();
        insert(&mut fields, "a.b[0].c[2]".into(), Value::Integer(10));

        let expected = fields_from_json(json!({
            "a": {
                "b": [{
                    "c": [null, null, 10]
                }]
            }
        }));
        assert_eq!(fields, expected);
    }
}