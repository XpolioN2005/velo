use crate::core::*;

pub fn resolve_string(input: &str, ctx: &Context) -> String {
    let mut out = input.to_string();

    // {0}, {1}, ...
    for (i, arg) in ctx.args.iter().enumerate() {
        let key = format!("{{{}}}", i);
        out = out.replace(&key, arg);
    }

    // {var:name}
    for (k, v) in &ctx.vars {
        let key = format!("{{var:{}}}", k);
        let val = value_to_string(v);
        out = out.replace(&key, &val);
    }

    out
}

pub fn resolve_args(args: &[String], ctx: &Context) -> Vec<String> {
    args.iter().map(|a| resolve_string(a, ctx)).collect()
}

pub fn value_to_string(v: &Value) -> String {
    match v {
        Value::None => "".into(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
    }
}
