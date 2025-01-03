pub mod tag;
use ssh2_config::Host;

#[derive(Debug)]
pub struct EnhancedHost {
    pub host: Host,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]

        let value: Value = group.into();

        if let Value::Object(obj) = value {
            assert_eq!(
                obj.get("name").and_then(|v| Some(v.clone().as_string())),
                Some("TestGroup".to_string())
            );

    }
}
