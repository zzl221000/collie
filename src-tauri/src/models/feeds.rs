use core::fmt;
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use chrono::{DateTime, FixedOffset, Utc};
use rusqlite::{Result, Row};
use sea_query::{Expr, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::{Deserialize, Serialize};

use super::database::{open_connection, Feeds};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum FeedStatus {
    Subscribed,
    Unsubscribed,
}

impl Display for FeedStatus {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            FeedStatus::Subscribed => write!(f, "subscribed"),
            FeedStatus::Unsubscribed => write!(f, "unsubscribed"),
        }
    }
}

impl FromStr for FeedStatus {
    type Err = ();

    fn from_str(x: &str) -> Result<FeedStatus, Self::Err> {
        match x {
            "subscribed" => Ok(FeedStatus::Subscribed),
            "unsubscribed" => Ok(FeedStatus::Unsubscribed),
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Feed {
    pub id: i32,
    pub title: String,
    pub link: String,
    pub status: FeedStatus,
    pub checked_at: DateTime<FixedOffset>,
}

impl From<&Row<'_>> for Feed {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get_unwrap("id"),
            title: row.get_unwrap("title"),
            link: row.get_unwrap("link"),
            status: FeedStatus::from_str(&row.get_unwrap::<&str, String>("status")).unwrap(),
            checked_at: row.get_unwrap("checked_at"),
        }
    }
}

#[derive(Deserialize)]
pub struct FeedToCreate {
    pub title: String,
    pub link: String,
}

#[derive(Deserialize)]
pub struct FeedToUpdate {
    pub id: i32,
    pub title: Option<String>,
    pub link: Option<String>,
    pub status: Option<FeedStatus>,
    pub checked_at: Option<DateTime<FixedOffset>>,
}

pub fn create(arg: FeedToCreate) -> Result<usize> {
    let db = open_connection()?;

    let cols = [Feeds::Title, Feeds::Link, Feeds::CheckedAt];
    let vals = [arg.title.into(), arg.link.into(), Utc::now().into()];
    let (sql, values) = Query::insert()
        .into_table(Feeds::Table)
        .columns(cols)
        .values_panic(vals)
        .build_rusqlite(SqliteQueryBuilder);

    db.execute(sql.as_str(), &*values.as_params())
}

pub fn read_all() -> Result<Vec<Feed>> {
    let db = open_connection()?;

    let cols = [
        Feeds::Id,
        Feeds::Title,
        Feeds::Link,
        Feeds::Status,
        Feeds::CheckedAt,
    ];
    let (sql, values) = Query::select()
        .columns(cols)
        .from(Feeds::Table)
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db.prepare(sql.as_str())?;
    let rows = stmt.query_map(&*values.as_params(), |x| Ok(Feed::from(x)))?;

    Ok(rows.map(|x| x.unwrap()).collect::<Vec<Feed>>())
}

pub fn read(id: i32) -> Result<Option<Feed>> {
    let db = open_connection()?;

    let (sql, values) = Query::select()
        .columns([
            Feeds::Id,
            Feeds::Title,
            Feeds::Link,
            Feeds::Status,
            Feeds::CheckedAt,
        ])
        .from(Feeds::Table)
        .and_where(Expr::col(Feeds::Id).eq(id))
        .limit(1)
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = db.prepare(sql.as_str())?;
    let mut rows = stmt.query(&*values.as_params())?;

    Ok(rows.next()?.map(Feed::from))
}

pub fn update(arg: &FeedToUpdate) -> Result<usize> {
    let db = open_connection()?;

    let mut vals = vec![];
    if let Some(title) = &arg.title {
        vals.push((Feeds::Title, title.into()));
    }
    if let Some(link) = &arg.link {
        vals.push((Feeds::Link, link.into()));
    }
    if let Some(status) = &arg.status {
        vals.push((Feeds::Status, status.to_string().into()));
    }
    if let Some(checked_at) = arg.checked_at {
        vals.push((Feeds::CheckedAt, checked_at.into()));
    }

    let (sql, values) = Query::update()
        .table(Feeds::Table)
        .values(vals)
        .and_where(Expr::col(Feeds::Id).eq(arg.id))
        .build_rusqlite(SqliteQueryBuilder);

    db.execute(sql.as_str(), &*values.as_params())
}

pub fn delete(id: i32) -> Result<usize> {
    let db = open_connection()?;

    let (sql, values) = Query::delete()
        .from_table(Feeds::Table)
        .and_where(Expr::col(Feeds::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);

    db.execute(sql.as_str(), &*values.as_params())
}
