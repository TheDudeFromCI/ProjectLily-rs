use std::env;

use chrono::NaiveDateTime;
use rusqlite::{Connection, Result};

use super::MemoryDBError;
use crate::prompt::ChatMessage;

pub fn open_connection() -> Result<Connection, MemoryDBError> {
    let url = env::var("DATABASE_URL").map_err(|_| MemoryDBError::DatabaseUrlNotSet)?;
    let conn = Connection::open(url)?;

    conn.execute_batch(
        r#"
        BEGIN;
        CREATE TABLE IF NOT EXISTS chat_log (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            role VARCHAR(255) NOT NULL,
            user VARCHAR(255),
            content TEXT NOT NULL,
            action VARCHAR(255),
            severity VARCHAR(255),
            time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            token_count INT2 NOT NULL
        );
        COMMIT;
        "#,
    )?;

    Ok(conn)
}

pub fn get_recent_logs(
    conn: &Connection,
    max_tokens: usize,
) -> Result<Vec<ChatMessage>, MemoryDBError> {
    let mut statement = conn.prepare(
        r#"
        SELECT role, user, content, action, severity, time, token_count
        FROM chat_log AS t1
        WHERE (
            SELECT SUM(token_count)
            FROM chat_log AS t2
            WHERE t2.time >= t1.time
        ) <= ?1
        ORDER BY time DESC
        "#,
    )?;

    let mut rows = statement.query([max_tokens])?;

    let mut messages = Vec::new();
    while let Some(row) = rows.next()? {
        let role: String = row.get(0)?;
        let user: Option<String> = row.get(1)?;
        let content: String = row.get(2)?;
        let action: Option<String> = row.get(3)?;
        let severity: Option<String> = row.get(4)?;
        let _time: NaiveDateTime = row.get(5)?;
        let token_count: i16 = row.get(6)?;

        let message = match role.as_str() {
            "system" => ChatMessage::System {
                severity: severity.unwrap().parse().unwrap(),
                content,
                tokens: Some(token_count as usize),
            },
            "user" => ChatMessage::User {
                username: user.unwrap(),
                content,
                tokens: Some(token_count as usize),
            },
            "assistant" => ChatMessage::Assistant {
                action: action.unwrap().parse().unwrap(),
                content,
                tokens: Some(token_count as usize),
            },
            _ => {
                return Err(MemoryDBError::UnexpectedDatabaseEntry {
                    expected: "'system', 'user', or 'assistant'".to_string(),
                    actual: role,
                });
            }
        };
        messages.push(message);
    }

    Ok(messages)
}

pub fn append_to_log(conn: &Connection, message: &ChatMessage) -> Result<(), MemoryDBError> {
    let mut statement = conn.prepare(
        r#"
        INSERT INTO chat_log (role, user, content, action, severity, token_count)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
    )?;

    match message {
        ChatMessage::System {
            severity,
            content,
            tokens,
        } => {
            statement.execute((
                "system",
                None::<String>,
                content,
                None::<String>,
                Some(severity.to_string()),
                tokens,
            ))?;
        }
        ChatMessage::User {
            username,
            content,
            tokens,
        } => {
            statement.execute((
                "user",
                Some(username.to_string()),
                content,
                None::<String>,
                None::<String>,
                tokens,
            ))?;
        }
        ChatMessage::Assistant {
            action,
            content,
            tokens,
        } => {
            statement.execute((
                "assistant",
                None::<String>,
                content,
                Some(action.to_string()),
                None::<String>,
                tokens,
            ))?;
        }
    };

    Ok(())
}
