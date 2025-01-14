use serde::{Deserialize, Serialize};
use surrealdb::{sql::Thing, Connection, Error, Response, Surreal};

pub trait TableName: 'static {
    const TABLE_NAME: &'static str;
}

#[derive(Debug)]
pub struct TagTable;
#[derive(Debug)]
pub struct GroupTable;

impl TableName for TagTable {
    const TABLE_NAME: &'static str = "tag";
}

impl TableName for GroupTable {
    const TABLE_NAME: &'static str = "group";
}

#[derive(Debug, Deserialize, Serialize)]
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
    ) -> surrealdb::Result<Option<Thing>> {
        #[derive(Debug, Deserialize)]
        struct Record {
            id: Thing,
        }

        let record: Option<Record> = db
            .query(format!(
                "SELECT id FROM {} WHERE name = $name LIMIT 1",
                T::TABLE_NAME
            ))
            .bind(("name", name.clone().to_lowercase()))
            .await
            .unwrap()
            .take(0)?;

        match record {
            Some(r) => Ok(Some(r.id)),
            None => Ok(None),
        }
    }

    pub async fn update<C: Connection>(db: &Surreal<C>, data: Record<T>) -> Result<Thing, Error>
    where
        T: 'static,
    {
        let record_id = match Self::get_id_by_name(db, data.name.clone()).await {
            Ok(t) => match t {
                Some(r) => r,
                None => todo!(),
            },
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
            Ok(t) => match t {
                Some(r) => r,
                None => todo!(),
            },
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
            None => {
                let tag = Self::create(db, name).await?;
                Ok(tag)
            }
        }
    }

    pub async fn add_relation<C: Connection>(
        db: &Surreal<C>,
        host_id: &Thing,
        record_id: &Thing,
    ) -> Result<Response, Error>
    where
        T: TableName,
    {
        let relation_table = match T::TABLE_NAME {
            "tag" => "tagged",
            "group" => "groupped",
            _ => {
                return Err(Error::Db(surrealdb::error::Db::InvalidModel {
                    message: String::from("Invalid table name"),
                }))
            }
        };

        match db
            .query(format!(
                "RELATE {}->{}->{}",
                record_id, relation_table, host_id
            ))
            .await
        {
            Ok(r) => Ok(r),
            Err(e) => Err(e),
        }
    }
    pub async fn remove_relation<C: Connection>(
        db: &Surreal<C>,
        host_id: &Thing,
        record_id: &Thing,
    ) -> Result<(), Error>
    where
        T: TableName,
    {
        let relation_table = match T::TABLE_NAME {
            "tag" => "tagged",
            "group" => "groupped",
            _ => {
                return Err(Error::Db(surrealdb::error::Db::InvalidModel {
                    message: String::from("Invalid table name"),
                }))
            }
        };

        let res = db
            .query(format!(
                "DELETE FROM {} WHERE in = {} AND out = {}",
                relation_table, record_id, host_id
            ))
            .await;

        res.unwrap();

        Ok(())
    }
}

pub type Tag = Record<TagTable>;
pub type Group = Record<GroupTable>;
