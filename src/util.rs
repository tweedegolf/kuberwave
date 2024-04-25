use crate::error::{Error, ErrorKind, Result};

pub fn map_value(x: &yaml_rust::Yaml) -> serde_json::value::Value {
    use serde_json::value::Value;
    use yaml_rust::Yaml;

    match x {
        Yaml::Real(str) => {
            Value::Number(serde_json::Number::from_f64(str.parse::<f64>().unwrap()).unwrap())
        }
        Yaml::Integer(i) => Value::Number(serde_json::Number::from(*i)),
        Yaml::String(str) => Value::String(str.to_owned()),
        Yaml::Boolean(b) => Value::Bool(*b),
        Yaml::Array(a) => Value::Array(a.iter().map(map_value).collect()),
        Yaml::Hash(h) => Value::Object(
            h.into_iter()
                .map(|(k, v)| (k.as_str().unwrap().to_owned(), map_value(v)))
                .collect(),
        ),
        Yaml::Alias(_) => unimplemented!(),
        Yaml::Null => Value::Null,
        Yaml::BadValue => unimplemented!(),
    }
}

pub fn append_yaml_to_context(xs: Vec<yaml_rust::Yaml>, context: &mut tera::Context) -> Result<()> {
    for range in xs {
        for (k, v) in map_value(&range)
            .as_object()
            .ok_or(ErrorKind::ContextError)?
        {
            context.insert(k, v);
        }
    }

    Ok(())
}

pub fn map_yaml_to_context(xs: Vec<yaml_rust::Yaml>) -> Result<tera::Context> {
    let mut context = tera::Context::new();
    append_yaml_to_context(xs, &mut context)?;
    Ok(context)
}

pub fn process_template(
    path: &std::path::Path,
    context: &tera::Context,
) -> Result<std::string::String> {
    use failure::ResultExt;

    tera::Tera::one_off(
        &std::fs::read_to_string(path).context(ErrorKind::FileReadError {
            name: path.to_owned(),
        })?,
        context,
        false,
    )
    .map_err(|e| {
        Error::create(
            e.to_string(),
            ErrorKind::TemplateError {
                name: path.to_owned(),
            },
        )
    })
}

pub fn get_secret() -> Option<String> {
    std::env::var("SECRET").ok()
}
