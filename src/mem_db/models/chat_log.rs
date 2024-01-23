use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Integer, Nullable, SmallInt, Text, Timestamp};

use crate::prompt::ChatMessage;

#[derive(Insertable)]
#[diesel(table_name = crate::schema::chat_log)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct NewChatMessage<'a> {
    pub role: &'static str,
    pub user: Option<&'a str>,
    pub content: &'a str,
    pub action: Option<&'static str>,
    pub severity: Option<&'static str>,
    pub time: NaiveDateTime,
    pub token_count: i16,
}

impl<'a> NewChatMessage<'a> {
    fn new(msg: &'a ChatMessage) -> Self {
        match msg {
            ChatMessage::System {
                severity,
                content,
                tokens,
            } => Self {
                role: msg.get_role(),
                user: None,
                content,
                action: None,
                severity: Some(severity.name()),
                time: chrono::Local::now().naive_local(),
                token_count: tokens.unwrap_or(0) as i16,
            },
            ChatMessage::User {
                username,
                content,
                tokens,
            } => Self {
                role: msg.get_role(),
                user: Some(username),
                content,
                action: None,
                severity: None,
                time: chrono::Local::now().naive_local(),
                token_count: tokens.unwrap_or(0) as i16,
            },
            ChatMessage::Assistant {
                action,
                content,
                tokens,
            } => Self {
                role: msg.get_role(),
                user: None,
                content,
                action: Some(action.name()),
                severity: None,
                time: chrono::Local::now().naive_local(),
                token_count: tokens.unwrap_or(0) as i16,
            },
        }
    }
}

#[derive(QueryableByName, Queryable, Selectable)]
#[diesel(table_name = crate::schema::chat_log)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct ChatLogEntry {
    pub id: i32,
    pub role: String,
    pub user: Option<String>,
    pub content: String,
    pub action: Option<String>,
    pub severity: Option<String>,
    pub time: NaiveDateTime,
    pub token_count: i16,
}

impl From<ChatLogEntry> for ChatMessage {
    fn from(value: ChatLogEntry) -> Self {
        match value.role.as_str() {
            "system" => ChatMessage::System {
                severity: value.severity.unwrap().parse().unwrap(),
                content: value.content,
                tokens: Some(value.token_count as usize),
            },
            "user" => ChatMessage::User {
                username: value.user.unwrap(),
                content: value.content,
                tokens: Some(value.token_count as usize),
            },
            "assistant" => ChatMessage::Assistant {
                action: value.action.unwrap().parse().unwrap(),
                content: value.content,
                tokens: Some(value.token_count as usize),
            },
            _ => panic!("Invalid role"),
        }
    }
}

pub struct ChatLog;

impl ChatLog {
    pub fn insert(msg: &ChatMessage, conn: &mut SqliteConnection) -> QueryResult<usize> {
        use crate::schema::chat_log::dsl::*;

        let record = NewChatMessage::new(msg);
        diesel::insert_into(chat_log).values(&record).execute(conn)
    }

    pub fn get_recent(
        max_tokens: usize,
        conn: &mut SqliteConnection,
    ) -> QueryResult<Vec<ChatMessage>> {
        use crate::schema::chat_log::dsl::*;

        // I tried to do this with Diesel, but I'm too dump to figure it out.
        // Raw SQL it is.
        // TODO: Figure out how to do this with Diesel.

        // let t1 = chat_log.select(ChatLogEntry::as_select());
        // let t2 = chat_log.select(sum(token_count));
        // let t2 = t2.filter(time.ge(time));

        // let results = t1
        //     .filter(t2.single_value().le(max_tokens as i64))
        //     .order(time.desc())
        //     .load::<ChatLogEntry>(conn)?;

        let results = sql_query(format!(
            r#"
            SELECT id, role, user, content, action, severity, time, token_count
            FROM chat_log AS t1
            WHERE (
                SELECT SUM(tokens)
                FROM chat_log AS t2
                WHERE t2.time >= t1.time
            ) <= {}
            ORDER BY time DESC
            "#,
            max_tokens
        ))
        .bind::<Integer, _>(id)
        .bind::<Text, _>(role)
        .bind::<Nullable<Text>, _>(user)
        .bind::<Text, _>(content)
        .bind::<Nullable<Text>, _>(action)
        .bind::<Nullable<Text>, _>(severity)
        .bind::<Timestamp, _>(time)
        .bind::<SmallInt, _>(token_count)
        .load::<ChatLogEntry>(conn)?;

        Ok(results.into_iter().map(|e| e.into()).collect())
    }
}
