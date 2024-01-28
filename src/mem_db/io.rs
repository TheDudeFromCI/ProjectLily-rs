use std::env;
use std::io::Write;

use chrono::NaiveDateTime;
use rusqlite::ffi::sqlite3_auto_extension;
use rusqlite::{Connection, DatabaseName, Result};
use sqlite_vss::{sqlite3_vector_init, sqlite3_vss_init};

use super::embedding::{MemoryEmbedding, VectorMemory, EMBEDDING_DIM};
use super::{MemoryDBError, RecalledMemory};
use crate::prompt::ChatMessage;

pub fn open_connection() -> Result<Connection, MemoryDBError> {
    let url = env::var("DATABASE_URL").map_err(|_| MemoryDBError::DatabaseUrlNotSet)?;
    let conn = Connection::open(url)?;

    unsafe {
        sqlite3_auto_extension(Some(sqlite3_vector_init));
        sqlite3_auto_extension(Some(sqlite3_vss_init));
    }

    conn.execute("BEGIN", [])?;

    conn.execute(
        r#"
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
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS vector_mem (
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            cached BOOLEAN NOT NULL DEFAULT FALSE,
            text TEXT NOT NULL,
            embedding BLOB NOT NULL
        );
        "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS vss_vector_mem USING vss0 (
            embedding(?1),
        );
        "#,
        [EMBEDDING_DIM as i32],
    )?;

    conn.execute("COMMIT", [])?;

    insert_cache_vector_mem(&conn)?;

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

pub fn save_vector_mem(conn: &Connection, memory: &VectorMemory) -> Result<(), MemoryDBError> {
    let mut statement = conn.prepare(
        r#"
        INSERT INTO vector_mem (text, embedding)
        VALUES (?1, ZEROBLOB(?2))
        "#,
    )?;
    statement.execute((memory.text(), EMBEDDING_DIM))?;
    let rowid = conn.last_insert_rowid();

    let mut blob = conn.blob_open(DatabaseName::Main, "vector_mem", "embedding", rowid, false)?;
    blob.write_all(&memory.embedding().as_bytes())?;

    let mut statement = conn.prepare(
        r#"
        INSERT INTO vss_vector_mem (rowid, text, embedding)
        FROM vector_mem
        "#,
    )?;
    statement.execute([rowid])?;

    Ok(())
}

fn insert_cache_vector_mem(conn: &Connection) -> Result<(), MemoryDBError> {
    let vector = VectorMemory::new(String::from(""), MemoryEmbedding::empty());

    let mut statement = conn.prepare(
        r#"
        INSERT INTO vector_mem (cached, text, embedding)
        VALUES (TRUE, "", ZEROBLOB(?1))
        "#,
    )?;
    statement.execute([EMBEDDING_DIM])?;
    let rowid = conn.last_insert_rowid();

    let mut blob = conn.blob_open(DatabaseName::Main, "vector_mem", "embedding", rowid, false)?;
    blob.write_all(&vector.embedding().as_bytes())?;

    Ok(())
}

fn replace_cache_vector_mem(
    conn: &Connection,
    embed: &MemoryEmbedding,
) -> Result<(), MemoryDBError> {
    let mut statement = conn.prepare(
        r#"
        SELECT rowid
        WHERE cached = TRUE
        LIMIT 1
        "#,
    )?;
    let rowid = statement.query_row([], |row| row.get(0))?;

    let mut blob = conn.blob_open(DatabaseName::Main, "vector_mem", "embedding", rowid, false)?;
    blob.write_all(&embed.as_bytes())?;

    Ok(())
}

pub fn recall_memories(
    conn: &Connection,
    search: &MemoryEmbedding,
    count: usize,
    max_distance: f32,
) -> Result<Vec<RecalledMemory>, MemoryDBError> {
    replace_cache_vector_mem(conn, search)?;

    let mut statement = conn.prepare(
        r#"
        WITH matches AS (
            SELECT rowid, distance
            FROM vss_vector_mem
            WHERE vss_search(
                embedding,
                (
                    SELECT embedding
                    FROM vector_mem
                    WHERE cached = TRUE
                    LIMIT 1
                )
            )
            LIMIT ?1
        )
        SELECT vector_mem.text, matches.distance
        FROM matches
        LEFT JOIN vector_mem ON vector_mem.rowid = matches.rowid
        WHERE matches.distance <= ?2
        "#,
    )?;

    let mut rows = statement.query((count as i32, max_distance))?;
    let mut memories = Vec::new();

    while let Some(row) = rows.next()? {
        let memory: String = row.get(0)?;
        let distance: f32 = row.get(1)?;
        memories.push(RecalledMemory { memory, distance });
    }

    Ok(memories)
}
