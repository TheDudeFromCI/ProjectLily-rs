use chrono::NaiveDateTime;
use diesel::prelude::*;

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

pub struct ChatLog;

impl ChatLog {
    pub fn insert(msg: &ChatMessage, conn: &mut SqliteConnection) -> QueryResult<usize> {
        use crate::schema::chat_log::dsl::*;

        let record = NewChatMessage::new(msg);
        diesel::insert_into(chat_log).values(&record).execute(conn)
    }
}
