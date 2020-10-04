use anyhow::{Error, Result};
use hocon::{Hocon, HoconLoader};
use serde_json::{Number, Value};

pub fn read_params(file: &str) -> Result<Value> {
    let hocon = HoconLoader::new().load_file(&file)?.hocon()?;
    hocon_to_json(hocon).ok_or_else(|| {
        Error::msg(format!(
            "Failed to convert config file '{}' to JSON format",
            file
        ))
    })
}

fn hocon_to_json(hocon: Hocon) -> Option<Value> {
    match hocon {
        Hocon::Boolean(b) => Some(Value::Bool(b)),
        Hocon::Integer(i) => Some(Value::Number(Number::from(i))),
        Hocon::Real(f) => Some(Value::Number(
            Number::from_f64(f).unwrap_or_else(|| Number::from(0)),
        )),
        Hocon::String(s) => Some(Value::String(s)),
        Hocon::Array(vec) => Some(Value::Array(
            vec.into_iter()
                .map(hocon_to_json)
                .filter_map(|i| i)
                .collect(),
        )),
        Hocon::Hash(map) => Some(Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, hocon_to_json(v)))
                .filter_map(|(k, v)| v.map(|v| (k, v)))
                .collect(),
        )),
        Hocon::Null => Some(Value::Null),
        Hocon::BadValue(_) => None,
    }
}
