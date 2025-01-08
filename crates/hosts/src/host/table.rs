use serde::{Deserialize, Serialize};
use surrealdb::{sql::Thing, Connection, Error, Surreal};

pub trait TableName: 'static {
    const TABLE_NAME: &'static str;
}

pub struct TagTable;
pub struct GroupTable;

impl TableName for TagTable {
    const TABLE_NAME: &'static str = "tag";
}

impl TableName for GroupTable {
    const TABLE_NAME: &'static str = "group";
}

#[derive(Deserialize, Serialize)]
pub struct Record<T: TableName> {
    pub name: String,
    #[serde(skip)]
    _marker: std::marker::PhantomData<T>,
}

impl<T: TableName> Record<T> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn get_id_by_name<C: Connection>(
        db: &Surreal<C>,
        name: String,
    ) -> surrealdb::Result<Thing> {
        let tag_id: Option<Thing> = db
            .query(format!(
                "SELECT id FROM {} WHERE name = $name LIMIT 1",
                T::TABLE_NAME
            ))
            .bind(("name", name.clone()))
            .await?
            .take(0)?;

        println!("{:#?}", tag_id);

        tag_id.ok_or(Error::Db(surrealdb::error::Db::InvalidModel {
            message: String::from("Tag not found"),
        }))
    }

    pub async fn update<C: Connection>(db: &Surreal<C>, data: Record<T>) -> Result<Thing, Error>
    where
        T: 'static,
    {
        let record_id = match Self::get_id_by_name(db, data.name.clone()).await {
            Ok(t) => t,
            Err(e) => return Err(e),
        };
        let _: Option<Record<T>> = db
            .update((T::TABLE_NAME, &record_id.id.to_string()))
            .content(data)
            .await?;
        Ok(record_id)
    }

    pub async fn create<C: Connection>(db: &Surreal<C>, name: String) -> Result<Thing, Error> {
        let created: Option<Record<T>> = db
            .create(T::TABLE_NAME)
            .content(Self {
                name: name.clone(),
                _marker: std::marker::PhantomData,
            })
            .await?;

        let _ = created.ok_or(Error::Db(surrealdb::error::Db::InvalidModel {
            message: String::from("Failed to create new tag"),
        }))?;
        let record_id = match Self::get_id_by_name(db, name).await {
            Ok(t) => t,
            Err(e) => return Err(e),
        };
        Ok(record_id)
    }

    pub async fn create_or_update<C: Connection>(
        name: String,
        db: &Surreal<C>,
    ) -> surrealdb::Result<Thing> {
        let existing: Option<Record<T>> = db
            .query(format!(
                "SELECT * FROM {} WHERE name = $name LIMIT 1",
                T::TABLE_NAME,
            ))
            .bind(("name", name.clone()))
            .await?
            .take(0)?;

        match existing {
            // If tag exists, update it
            Some(tag) => Self::update(db, tag).await,
            // If tag doesn't exist, create new one
            None => Self::create(db, name).await,
        }
    }
}

pub type Tag = Record<TagTable>;
pub type Group = Record<GroupTable>;
