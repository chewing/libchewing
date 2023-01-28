use std::{path::Path, str};

use rusqlite::{params, Connection, Error as RusqliteError, OpenFlags, OptionalExtension};
use thiserror::Error;

use crate::zhuyin::{IntoSyllablesBytes, Syllable};

use super::{
    BuildDictionaryError, DictEntries, Dictionary, DictionaryBuilder, DictionaryInfo,
    DictionaryMut, DictionaryUpdateError, Phrase, Phrases,
};

#[derive(Debug, Error)]
#[error("sqlite error")]
pub enum SqliteDictionaryError {
    SqliteError {
        #[from]
        source: RusqliteError,
    },
    MissingTable {
        table: String,
    },
}

#[derive(Debug)]
pub struct SqliteDictionary {
    conn: Connection,
    info: DictionaryInfo,
    read_only: bool,
}

impl SqliteDictionary {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<SqliteDictionary, SqliteDictionaryError> {
        let mut conn = Connection::open(path)?;
        Self::initialize_tables(&conn)?;
        Self::migrate_from_userphrase_v1(&mut conn)?;
        Self::ensure_tables(&conn)?;
        let info = Self::read_info_v1(&conn)?;

        Ok(SqliteDictionary {
            conn,
            info,
            read_only: false,
        })
    }

    pub fn open_read_only<P: AsRef<Path>>(
        path: P,
    ) -> Result<SqliteDictionary, SqliteDictionaryError> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        Self::ensure_tables(&conn)?;
        let info = Self::read_info_v1(&conn)?;

        Ok(SqliteDictionary {
            conn,
            info,
            read_only: true,
        })
    }

    pub fn open_in_memory() -> Result<SqliteDictionary, SqliteDictionaryError> {
        let conn = Connection::open_in_memory()?;
        Self::initialize_tables(&conn)?;
        Self::ensure_tables(&conn)?;
        let info = Self::read_info_v1(&conn)?;

        Ok(SqliteDictionary {
            conn,
            info,
            read_only: false,
        })
    }

    fn initialize_tables(conn: &Connection) -> Result<(), SqliteDictionaryError> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS dictionary_v1 (
                syllables BLOB NOT NULL,
                phrase TEXT NOT NULL,
                freq INTEGER NOT NULL,
                sort_id INTEGER,
                userphrase_id INTEGER,
                PRIMARY KEY (syllables, phrase)
            ) WITHOUT ROWID",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS userphrase_v2 (
                id INTEGER PRIMARY KEY,
                user_freq INTEGER,
                time INTEGER
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS migration_v1 (name TEXT PRIMARY KEY) WITHOUT ROWID",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS info_v1 (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            ) WITHOUT ROWID",
            [],
        )?;

        Ok(())
    }

    fn ensure_tables(conn: &Connection) -> Result<(), SqliteDictionaryError> {
        let mut stmt = conn
            .prepare("SELECT EXISTS (SELECT 1 FROM sqlite_schema WHERE type='table' AND name=?)")?;
        for table_name in ["dictionary_v1", "userphrase_v2", "migration_v1", "info_v1"] {
            let has_table: bool = stmt.query_row([table_name], |row| row.get(0))?;
            if !has_table {
                return Err(SqliteDictionaryError::MissingTable {
                    table: table_name.into(),
                });
            }
        }
        Ok(())
    }

    fn migrate_from_userphrase_v1(conn: &mut Connection) -> Result<(), SqliteDictionaryError> {
        let has_userphrase_v1: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='userphrase_v1')",
            [],
            |row| row.get(0)
        )?;
        let migrated: bool = conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM migration_v1 WHERE name='migrate_from_userphrase_v1')",
            [],
            |row| row.get(0),
        )?;
        if !has_userphrase_v1 || migrated {
            // Don't need to migrate
            conn.execute(
                "INSERT OR IGNORE INTO migration_v1 (name) VALUES ('migrate_from_userphrase_v1')",
                [],
            )?;
            return Ok(());
        }

        let mut userphrases: Vec<(Vec<Syllable>, String, u32, u32, u64)> = vec![];
        {
            let mut stmt = conn.prepare(
                "SELECT
                    phrase,
                    orig_freq,
                    user_freq,
                    time,
                    phone_0,
                    phone_1,
                    phone_2,
                    phone_3,
                    phone_4,
                    phone_5,
                    phone_6,
                    phone_7,
                    phone_8,
                    phone_9,
                    phone_10
                FROM userphrase_v1",
            )?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let mut syllables = vec![];
                for i in 4..15 {
                    let syllable_u16: u16 = row.get(i)?;
                    if let Ok(syllable) = Syllable::try_from(syllable_u16) {
                        if !syllable.is_empty() {
                            syllables.push(syllable);
                        }
                    }
                }
                userphrases.push((
                    syllables,
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                ));
            }
        }

        let tx = conn.transaction()?;
        {
            for item in userphrases {
                let mut stmt = tx.prepare_cached(
                    "INSERT INTO userphrase_v2 (
                        user_freq,
                        time
                    ) VALUES (?, ?)",
                )?;
                stmt.execute(params![item.3, item.4])?;
                let row_id = tx.last_insert_rowid();
                let mut stmt = tx.prepare_cached(
                    "INSERT OR REPLACE INTO dictionary_v1 (
                        syllables,
                        phrase,
                        freq,
                        userphrase_id
                    ) VALUES (?, ?, ?, ?)",
                )?;
                let mut syllables_bytes = vec![];
                item.0
                    .into_iter()
                    .for_each(|syl| syllables_bytes.extend_from_slice(&syl.to_u16().to_le_bytes()));
                stmt.execute(params![syllables_bytes, item.1, item.2, row_id])?;
            }
            tx.execute(
                "INSERT INTO migration_v1 (name) VALUES ('migrate_from_userphrase_v1')",
                [],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    fn read_info_v1(conn: &Connection) -> Result<DictionaryInfo, SqliteDictionaryError> {
        let mut info = DictionaryInfo::default();
        let mut stmt = conn.prepare(
            "SELECT key, value FROM info_v1 WHERE key IN (
                'name',
                'copyright',
                'license',
                'version',
                'software'
            )",
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            match key.as_str() {
                "name" => info.name = Some(value),
                "copyright" => info.copyright = Some(value),
                "license" => info.license = Some(value),
                "version" => info.version = Some(value),
                "software" => info.software = Some(value),
                _ => (),
            }
        }
        Ok(info)
    }
}

impl Dictionary for SqliteDictionary {
    fn lookup_phrase(&self, syllables: &[Syllable]) -> Phrases {
        let syllables_bytes = syllables.into_syllables_bytes();
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT
                    phrase,
                    max(freq, coalesce(user_freq, 0)),
                    time
                FROM dictionary_v1 LEFT JOIN userphrase_v2 ON userphrase_id = id
                WHERE syllables = ?
                ORDER BY sort_id ASC, max(freq, coalesce(user_freq, 0)) DESC, phrase DESC",
            )
            .expect("SQL error");
        Box::new(
            stmt.query_map([syllables_bytes], |row| {
                let (phrase, freq, time): (String, _, _) = row.try_into()?;
                let mut phrase = Phrase::new(phrase, freq);
                if let Some(last_used) = time {
                    phrase = phrase.with_time(last_used);
                }
                Ok(phrase)
            })
            .unwrap()
            .map(|r| r.unwrap())
            .collect::<Vec<_>>()
            .into_iter(),
        )
    }

    fn entries(&self) -> DictEntries {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT syllables, phrase, max(freq, coalesce(user_freq, 0)), time
                FROM dictionary_v1 LEFT JOIN userphrase_v2 ON userphrase_id = id",
            )
            .expect("SQL error");
        Box::new(
            stmt.query_map([], |row| {
                let (syllables_bytes, phrase, freq, time): (Vec<u8>, String, _, _) =
                    row.try_into()?;
                let syllables = syllables_bytes
                    .chunks_exact(2)
                    .map(|bytes| {
                        let mut u16_bytes = [0; 2];
                        u16_bytes.copy_from_slice(bytes);
                        let syl_u16 = u16::from_le_bytes(u16_bytes);
                        Syllable::try_from(syl_u16).unwrap()
                    })
                    .collect::<Vec<_>>();
                let mut phrase = Phrase::new(phrase, freq);
                if let Some(last_used) = time {
                    phrase = phrase.with_time(last_used);
                }
                Ok((syllables, phrase))
            })
            .unwrap()
            .map(|r| r.unwrap())
            .collect::<Vec<_>>()
            .into_iter(),
        )
    }

    fn about(&self) -> DictionaryInfo {
        self.info.clone()
    }

    fn as_mut_dict(&mut self) -> Option<&mut dyn DictionaryMut> {
        if self.read_only {
            None
        } else {
            Some(self)
        }
    }
}

impl From<RusqliteError> for DictionaryUpdateError {
    fn from(source: RusqliteError) -> Self {
        DictionaryUpdateError {
            source: source.into(),
        }
    }
}

impl DictionaryMut for SqliteDictionary {
    fn insert(
        &mut self,
        syllables: &[Syllable],
        phrase: Phrase,
    ) -> Result<(), DictionaryUpdateError> {
        let syllables_bytes = syllables.into_syllables_bytes();
        let mut stmt = self.conn.prepare_cached(
            "INSERT OR REPLACE INTO dictionary_v1 (
                    syllables,
                    phrase,
                    freq
            ) VALUES (?, ?, ?)",
        )?;
        stmt.execute(params![syllables_bytes, phrase.as_str(), phrase.freq()])?;
        Ok(())
    }

    fn update(
        &mut self,
        syllables: &[Syllable],
        phrase: Phrase,
        user_freq: u32,
        time: u64,
    ) -> Result<(), DictionaryUpdateError> {
        let syllables_bytes = syllables.into_syllables_bytes();
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "SELECT userphrase_id FROM dictionary_v1 WHERE syllables = ? AND phrase = ?",
            )?;
            let userphrase_id: Option<Option<u64>> = stmt
                .query_row(params![syllables_bytes, phrase.as_str()], |row| row.get(0))
                .optional()?;
            match userphrase_id {
                Some(Some(id)) => {
                    let mut stmt =
                        tx.prepare_cached("UPDATE userphrase_v2 SET user_freq = ? WHERE id = ?")?;
                    stmt.execute(params![user_freq, id])?;
                }
                Some(None) | None => {
                    let mut stmt = tx.prepare_cached(
                        "INSERT INTO userphrase_v2 (user_freq, time) VALUES (?, ?)",
                    )?;
                    stmt.execute(params![user_freq, time])?;
                    let userphrase_id = tx.last_insert_rowid();
                    let mut stmt = tx.prepare_cached(
                        "INSERT OR REPLACE INTO dictionary_v1 (
                            syllables,
                            phrase,
                            freq,
                            userphrase_id
                        ) VALUES (?, ?, ?, ?)",
                    )?;
                    stmt.execute(params![
                        syllables_bytes,
                        phrase.as_str(),
                        phrase.freq(),
                        userphrase_id
                    ])?;
                }
            }
        }
        tx.commit()?;
        Ok(())
    }

    fn remove(
        &mut self,
        syllables: &[Syllable],
        phrase_str: &str,
    ) -> Result<(), DictionaryUpdateError> {
        let syllables_bytes = syllables.into_syllables_bytes();
        let mut stmt = self
            .conn
            .prepare_cached("DELETE FROM dictionary_v1 WHERE syllables = ? AND phrase = ?")?;
        stmt.execute(params![syllables_bytes, phrase_str])?;
        Ok(())
    }
}

pub struct SqliteDictionaryBuilder {
    dict: SqliteDictionary,
    sort_id: u64,
}

impl SqliteDictionaryBuilder {
    pub fn new() -> SqliteDictionaryBuilder {
        let dict = SqliteDictionary::open_in_memory().unwrap();
        SqliteDictionaryBuilder { dict, sort_id: 0 }
    }
}

impl Default for SqliteDictionaryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<RusqliteError> for BuildDictionaryError {
    fn from(source: RusqliteError) -> Self {
        BuildDictionaryError {
            source: Box::new(source),
        }
    }
}

impl From<std::str::Utf8Error> for BuildDictionaryError {
    fn from(source: std::str::Utf8Error) -> Self {
        BuildDictionaryError {
            source: Box::new(source),
        }
    }
}

impl DictionaryBuilder for SqliteDictionaryBuilder {
    fn set_info(&mut self, info: DictionaryInfo) -> Result<(), BuildDictionaryError> {
        let tx = self.dict.conn.transaction()?;
        {
            let mut stmt =
                tx.prepare("INSERT OR REPLACE INTO info_v1 (key, value) VALUES (?, ?)")?;
            if let Some(name) = info.name {
                stmt.execute(["name", &name])?;
            }
            if let Some(copyright) = info.copyright {
                stmt.execute(["copyright", &copyright])?;
            }
            if let Some(license) = info.license {
                stmt.execute(["license", &license])?;
            }
            if let Some(version) = info.version {
                stmt.execute(["version", &version])?;
            }
            if let Some(software) = info.software {
                stmt.execute(["software", &software])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    fn insert(
        &mut self,
        syllables: &[Syllable],
        phrase: Phrase,
    ) -> Result<(), BuildDictionaryError> {
        let sort_id = if syllables.len() == 1 {
            self.sort_id += 1;
            self.sort_id
        } else {
            0
        };
        let syllables_bytes = syllables.into_syllables_bytes();
        let mut stmt = self.dict.conn.prepare_cached(
            "INSERT OR REPLACE INTO dictionary_v1 (
                    syllables,
                    phrase,
                    freq,
                    sort_id
            ) VALUES (?, ?, ?, ?)",
        )?;
        stmt.execute(params![
            syllables_bytes,
            phrase.as_str(),
            phrase.freq(),
            sort_id
        ])?;

        Ok(())
    }

    fn build(&mut self, path: &Path) -> Result<(), BuildDictionaryError> {
        let path = path.to_str().ok_or(BuildDictionaryError {
            source: "cannot convert file path to utf8".into(),
        })?;
        self.dict.conn.execute("VACUUM INTO ?", [path])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use rusqlite::{params, Connection};
    use tempfile::NamedTempFile;

    use crate::{
        dictionary::{Dictionary, DictionaryMut, Phrase},
        syl,
        zhuyin::Bopomofo,
    };

    use super::SqliteDictionary;

    #[test]
    fn migration_from_userphrase_v1() {
        let temp_path = NamedTempFile::new()
            .expect("Unable to create tempfile")
            .into_temp_path();
        let temp_db = Connection::open(&temp_path).expect("Unable to open database");
        temp_db.execute(
            "CREATE TABLE IF NOT EXISTS userphrase_v1 (
                time INTEGER,
                user_freq INTEGER,
                max_freq INTEGER,
                orig_freq INTEGER,
                length INTEGER,
                phone_0 INTEGER,
                phone_1 INTEGER,
                phone_2 INTEGER,
                phone_3 INTEGER,
                phone_4 INTEGER,
                phone_5 INTEGER,
                phone_6 INTEGER,
                phone_7 INTEGER,
                phone_8 INTEGER,
                phone_9 INTEGER,
                phone_10 INTEGER,
                phrase TEXT,
                PRIMARY KEY (phone_0,phone_1,phone_2,phone_3,phone_4,phone_5,phone_6,phone_7,phone_8,phone_9,phone_10,phrase)
            )", []).expect("Initialize db failed");
        temp_db
            .execute(
                "INSERT INTO userphrase_v1 (
                    time, user_freq, max_freq, orig_freq, length,
                    phone_0,phone_1,phone_2,phone_3,phone_4,phone_5,phone_6,phone_7,phone_8,phone_9,phone_10,phrase
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?), (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![186613,9318,9318,9318,2,10268,8708,0,0,0,0,0,0,0,0,0,"測試".to_string(),
                        186613,318,9318,9318,2,10268,8708,0,0,0,0,0,0,0,0,0,"策士".to_string()],
            )
            .expect("Initialize db failed");
        temp_db.close().expect("Unable to close database");

        let dict = SqliteDictionary::open(&temp_path).expect("Unable to open database");
        assert_eq!(
            vec![
                Phrase::new("策士", 9318).with_time(186613),
                Phrase::new("測試", 9318).with_time(186613)
            ],
            dict.lookup_phrase(&[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ])
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn insert_and_update_user_freq() -> Result<(), Box<dyn Error>> {
        let mut dict = SqliteDictionary::open_in_memory()?;
        dict.update(
            &[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ],
            ("測試", 9318).into(),
            9900,
            0,
        )?;
        assert_eq!(
            vec![Phrase::new("測試", 9900).with_time(0)],
            dict.lookup_phrase(&[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ])
            .collect::<Vec<_>>()
        );
        Ok(())
    }

    #[test]
    fn update_user_freq() -> Result<(), Box<dyn Error>> {
        let mut dict = SqliteDictionary::open_in_memory()?;
        let syllables = [
            syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
            syl![Bopomofo::SH, Bopomofo::TONE4],
        ];
        dict.insert(&syllables, ("測試", 9318).into())?;
        dict.update(&syllables, ("測試", 9318).into(), 9900, 0)?;
        assert_eq!(
            vec![Phrase::new("測試", 9900).with_time(0)],
            dict.lookup_phrase(&[
                syl![Bopomofo::C, Bopomofo::E, Bopomofo::TONE4],
                syl![Bopomofo::SH, Bopomofo::TONE4],
            ])
            .collect::<Vec<_>>()
        );
        Ok(())
    }
}
