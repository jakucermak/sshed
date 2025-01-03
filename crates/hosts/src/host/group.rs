use serde::{Deserialize, Serialize};
use surrealdb::engine::local::Db;
use surrealdb::Result as SurrealResult;
use surrealdb::{
    sql::{Thing, Value},
    Surreal,
};
#[derive(Deserialize, Serialize)]
pub struct Group {
    pub id: Option<Thing>,
    pub name: String,
    pub hosts: Vec<Thing>,
}

impl From<Group> for Value {
    fn from(grp: Group) -> Self {
        let mut obj = surrealdb::sql::Object::default();
        obj.insert("name".into(), grp.name.into());
        obj.insert(
            "hosts".into(),
            Value::Array(grp.hosts.into_iter().map(Value::Thing).collect()),
        );
        Value::Object(obj)
    }
}

impl TryFrom<Value> for Group {
    type Error = surrealdb::Error;

    fn try_from(value: Value) -> SurrealResult<Self> {
        if let Value::Object(obj) = value {
            Ok(Group {
                id: obj.get("id").and_then(|v| match v {
                    Value::Thing(t) => Some(t.clone()),
                    _ => None,
                }),
                name: obj
                    .get("name")
                    .and_then(|v| match v {
                        Value::Strand(s) => Some(s.to_string()),
                        _ => None,
                    })
                    .unwrap_or_default(),
                hosts: obj
                    .get("hosts")
                    .and_then(|v| match v {
                        Value::Array(arr) => Some(
                            arr.iter()
                                .filter_map(|v| match v {
                                    Value::Thing(t) => Some(t.clone()),
                                    _ => None,
                                })
                                .collect(),
                        ),
                        _ => None,
                    })
                    .unwrap_or_default(),
            })
        } else {
            Err(surrealdb::Error::Db(surrealdb::error::Db::InvalidModel {
                message: String::from("Expected object"),
            }))
        }
    }
}

impl Group {
    /// Creates or updates a group and returns its Thing identifier
    /// Returns Ok(Thing) if successful, Err if database operation fails
    pub async fn create_or_update(
        name: String,
        hosts: Vec<Thing>,
        db: &Surreal<Db>,
    ) -> surrealdb::Result<Thing> {
        // First try to find existing group by name
        let existing: Option<Group> = db
            .query("SELECT * FROM group WHERE name = $name LIMIT 1")
            .bind(("name", name.clone()))
            .await?
            .take(0)?;

        match existing {
            // If group exists, update it
            Some(mut group) => {
                if let Some(id) = group.id.clone() {
                    group.hosts = hosts;
                    let _: Option<Group> = db
                        .update(("group", id.id.to_string()))
                        .content(group)
                        .await?;
                    Ok(id)
                } else {
                    Err(surrealdb::Error::Db(surrealdb::error::Db::InvalidModel {
                        message: String::from("Existing group has no ID"),
                    }))
                }
            }
            // If group doesn't exist, create new one
            None => {
                let created: Option<Group> = db
                    .create("group")
                    .content(Group {
                        id: None,
                        name,
                        hosts,
                    })
                    .await?;

                let new_group =
                    created.ok_or(surrealdb::Error::Db(surrealdb::error::Db::InvalidModel {
                        message: String::from("Failed to create new group"),
                    }))?;

                new_group
                    .id
                    .ok_or(surrealdb::Error::Db(surrealdb::error::Db::InvalidModel {
                        message: String::from("Created group has no ID"),
                    }))
            }
        }
    }
}
