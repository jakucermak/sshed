use serde::{Deserialize, Serialize};
use surrealdb::engine::local::Db;
use surrealdb::{
    sql::{Thing, Value},
    Surreal,
};

use surrealdb::Result as SurrealResult;
#[derive(Deserialize, Serialize)]
pub struct Tag {
    pub id: Option<Thing>,
    pub name: String,
    pub hosts: Vec<Thing>,
}

impl From<Tag> for Value {
    fn from(tag: Tag) -> Self {
        let mut obj = surrealdb::sql::Object::default();
        obj.insert("name".into(), tag.name.into());
        obj.insert(
            "hosts".into(),
            Value::Array(tag.hosts.into_iter().map(Value::Thing).collect()),
        );
        Value::Object(obj)
    }
}

impl TryFrom<Value> for Tag {
    type Error = surrealdb::Error;

    fn try_from(value: Value) -> SurrealResult<Self> {
        if let Value::Object(obj) = value {
            Ok(Tag {
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

impl Tag {
    pub async fn create_or_update(
        name: String,
        hosts: Vec<Thing>,
        db: &Surreal<Db>,
    ) -> surrealdb::Result<Thing> {
        // First try to find existing group by name
        let existing: Option<Tag> = db
            .query("SELECT * FROM group WHERE name = $name LIMIT 1")
            .bind(("name", name.clone()))
            .await?
            .take(0)?;

        match existing {
            // If group exists, update it
            Some(mut tag) => {
                if let Some(id) = tag.id.clone() {
                    tag.hosts = hosts;
                    let _: Option<Tag> = db.update(("tag", id.id.to_string())).content(tag).await?;
                    Ok(id)
                } else {
                    Err(surrealdb::Error::Db(surrealdb::error::Db::InvalidModel {
                        message: String::from("Existing group has no ID"),
                    }))
                }
            }
            // If group doesn't exist, create new one
            None => {
                let created: Option<Tag> = db
                    .create("group")
                    .content(Tag {
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
