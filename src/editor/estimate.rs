use std::path::Path;

use rusqlite::Connection;
use thiserror::Error;

use crate::dictionary::Phrase;

#[derive(Error, Debug)]
#[error("update estimate error")]
pub struct EstimateError {
    source: Box<dyn std::error::Error>,
}

pub trait UserFreqEstimate {
    fn tick(&mut self) -> Result<(), EstimateError>;
    fn now(&self) -> Result<u64, EstimateError>;
    fn estimate(&self, phrase: &Phrase, orig_freq: u32, max_freq: u32) -> u32;
}

pub struct SqliteUserFreqEstimate {
    conn: Connection,
    lifetime: u64,
}

impl From<rusqlite::Error> for EstimateError {
    fn from(source: rusqlite::Error) -> Self {
        EstimateError {
            source: source.into(),
        }
    }
}

impl SqliteUserFreqEstimate {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<SqliteUserFreqEstimate, EstimateError> {
        let conn = Connection::open(path)?;
        Self::initialize_tables(&conn)?;
        let lifetime = conn.query_row("SELECT value FROM config_v1 WHERE id = 0", [], |row| {
            row.get(0)
        })?;
        Ok(SqliteUserFreqEstimate { conn, lifetime })
    }

    pub fn open_in_memory() -> Result<SqliteUserFreqEstimate, EstimateError> {
        let conn = Connection::open_in_memory()?;
        Self::initialize_tables(&conn)?;
        let lifetime = conn.query_row("SELECT value FROM config_v1 WHERE id = 0", [], |row| {
            row.get(0)
        })?;
        Ok(SqliteUserFreqEstimate { conn, lifetime })
    }

    fn initialize_tables(conn: &Connection) -> Result<(), EstimateError> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS config_v1 (
                id INTEGER PRIMARY KEY,
                value INTEGER
            ) WITHOUT ROWID",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO config_v1 (id, value) VALUES (0, 0)",
            [],
        )?;
        Ok(())
    }
}

const SHORT_INCREASE_FREQ: u32 = 10;
const MEDIUM_INCREASE_FREQ: u32 = 5;
const LONG_DECREASE_FREQ: u32 = 10;
const MAX_USER_FREQ: u32 = 99999999;

impl UserFreqEstimate for SqliteUserFreqEstimate {
    fn tick(&mut self) -> Result<(), EstimateError> {
        // TODO debounce write
        self.conn
            .execute("UPDATE config_v1 SET value = value + 1 WHERE id = 0", [])?;
        self.lifetime =
            self.conn
                .query_row("SELECT value FROM config_v1 WHERE id = 0", [], |row| {
                    row.get(0)
                })?;
        Ok(())
    }

    fn now(&self) -> Result<u64, EstimateError> {
        Ok(self.lifetime)
    }

    fn estimate(&self, phrase: &Phrase, orig_freq: u32, max_freq: u32) -> u32 {
        let delta_time = self.lifetime - phrase.last_used().unwrap_or(self.lifetime);

        if delta_time < 4000 {
            let delta = if phrase.freq() >= max_freq {
                ((max_freq - orig_freq) / 5 + 1).min(SHORT_INCREASE_FREQ)
            } else {
                ((max_freq - orig_freq) / 5 + 1).max(SHORT_INCREASE_FREQ)
            };
            (phrase.freq() + delta).min(MAX_USER_FREQ)
        } else if delta_time < 50000 {
            let delta = if phrase.freq() >= max_freq {
                ((max_freq - orig_freq) / 10 + 1).min(MEDIUM_INCREASE_FREQ)
            } else {
                ((max_freq - orig_freq) / 10 + 1).max(MEDIUM_INCREASE_FREQ)
            };
            (phrase.freq() + delta).min(MAX_USER_FREQ)
        } else {
            let delta = ((phrase.freq() - orig_freq) / 5).max(LONG_DECREASE_FREQ);
            (phrase.freq() - delta).max(orig_freq)
        }
    }
}
