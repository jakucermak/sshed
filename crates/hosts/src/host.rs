pub mod group;
pub mod tag;
use ssh2_config::Host;
use surrealdb::sql::Thing;

#[derive(Debug)]
pub struct EnhancedHost {
    pub host: Host,
    pub comment: Option<String>,
    pub groups: Vec<Thing>,
    pub tags: Vec<Thing>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use group::Group;
    use surrealdb::sql::Value;

    #[test]
    fn test_group_serialization() {
        let group = Group {
            id: Some(Thing::from(("group", "test"))),
            name: "TestGroup".to_string(),
            hosts: vec![
                Thing::from(("host", "host1")),
                Thing::from(("host", "host2")),
            ],
        };

        let value: Value = group.into();

        if let Value::Object(obj) = value {
            assert_eq!(
                obj.get("name").and_then(|v| Some(v.clone().as_string())),
                Some("TestGroup".to_string())
            );

            if let Some(Value::Array(hosts)) = obj.get("hosts") {
                assert_eq!(hosts.len(), 2);
                assert!(matches!(hosts[0], Value::Thing(_)));
                assert!(matches!(hosts[1], Value::Thing(_)));
            } else {
                panic!("Hosts should be an array");
            }
        } else {
            panic!("Should be an object");
        }
    }
}
