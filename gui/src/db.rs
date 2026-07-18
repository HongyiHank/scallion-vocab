use rusqlite::{params, Connection, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::log;
use crate::model::Deck;

const DECK_COLORS: &[&str] = &[
    "#006c45", "#008757", "#279b57", "#4eb674",
    "#6dd391", "#a0f2ad", "#005232", "#003920",
];

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open() -> Result<Self> {
        let db_path = Self::resolve_path();

        if let Some(parent) = db_path.parent() {
            if !parent.is_dir() {
                log!("[DB::Open] creating directory: {:?}", parent);
                std::fs::create_dir_all(parent).map_err(|e| {
                    log!("[DB::Open] failed to create directory: {e}");
                    rusqlite::Error::InvalidPath(db_path.clone())
                })?;
            }
        }

        log!("[DB::Open] opening database at: {:?}", db_path);
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Self::init_tables(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn resolve_path() -> PathBuf {
        if let Some(base) = dirs::data_local_dir() {
            return base.join("scallion-vocab").join("scallion-vocab.db");
        }
        let android = PathBuf::from("/data/data/com.scallion.vocab/files/scallion-vocab.db");
        if android.parent().map_or(false, |p| p.is_dir()) {
            return android;
        }
        let tmp = std::env::temp_dir().join("scallion-vocab.db");
        if tmp.parent().map_or(false, |p| p.is_dir()) {
            return tmp;
        }
        PathBuf::from("scallion-vocab.db")
    }

    fn init_tables(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS decks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                color TEXT NOT NULL,
                is_folder INTEGER NOT NULL DEFAULT 0,
                parent_id INTEGER REFERENCES decks(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS words (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                deck_id INTEGER NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
                front TEXT NOT NULL,
                back TEXT NOT NULL,
                pos TEXT NOT NULL DEFAULT '',
                pron TEXT NOT NULL DEFAULT '',
                example TEXT NOT NULL DEFAULT '',
                synonym TEXT NOT NULL DEFAULT '',
                antonym TEXT NOT NULL DEFAULT '',
                tags TEXT NOT NULL DEFAULT '[]'
            );",
        )?;
        let has_is_folder: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('decks') WHERE name='is_folder'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0)
            > 0;
        if !has_is_folder {
            let _ = conn.execute_batch(
                "ALTER TABLE decks ADD COLUMN is_folder INTEGER NOT NULL DEFAULT 0;
                 ALTER TABLE decks ADD COLUMN parent_id INTEGER REFERENCES decks(id) ON DELETE CASCADE;",
            );
        }
        let word_cols = ["pos", "pron", "example", "synonym", "antonym", "tags"];
        for col in &word_cols {
            let sql = format!("ALTER TABLE words ADD COLUMN {col} TEXT NOT NULL DEFAULT ''");
            let _ = conn.execute(&sql, []);
        }
        let has_updated_at: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('decks') WHERE name='updated_at'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;
        if !has_updated_at {
            conn.execute_batch(
                "ALTER TABLE decks ADD COLUMN updated_at TEXT NOT NULL DEFAULT '';
                 UPDATE decks SET updated_at = datetime('now','localtime') WHERE updated_at = '';",
            )?;
        }
        Ok(())
    }

    fn next_color(conn: &Connection) -> Result<String> {
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM decks", [], |r| r.get(0))?;
        let idx = (count as usize) % DECK_COLORS.len();
        Ok(DECK_COLORS[idx].to_string())
    }

    pub fn create_deck(&self, name: &str, parent_id: Option<i64>) -> Result<Deck> {
        let conn = self.conn.lock().unwrap();
        let color = Self::next_color(&conn)?;
        conn.execute(
            "INSERT INTO decks (name, color, is_folder, parent_id, updated_at) VALUES (?1, ?2, 0, ?3, datetime('now','localtime'))",
            params![name, color, parent_id],
        )?;
        let id = conn.last_insert_rowid();
        let updated_at: String = conn.query_row(
            "SELECT updated_at FROM decks WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )?;
        Ok(Deck {
            id,
            name: name.to_string(),
            color,
            word_count: 0,
            is_folder: false,
            parent_id,
            updated_at: updated_at.chars().take(16).collect(),
        })
    }

    pub fn create_folder(&self, name: &str, parent_id: Option<i64>) -> Result<Deck> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO decks (name, color, is_folder, parent_id, updated_at) VALUES (?1, '', 1, ?2, datetime('now','localtime'))",
            params![name, parent_id],
        )?;
        let id = conn.last_insert_rowid();
        let updated_at: String = conn.query_row(
            "SELECT updated_at FROM decks WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )?;
        Ok(Deck {
            id,
            name: name.to_string(),
            color: String::new(),
            word_count: 0,
            is_folder: true,
            parent_id,
            updated_at: updated_at.chars().take(16).collect(),
        })
    }

    pub fn list_by_parent(&self, parent_id: Option<i64>) -> Result<Vec<Deck>> {
        let conn = self.conn.lock().unwrap();
        // 資料夾 updated_at = subtree 中最新牌組的 updated_at；資料夾本身的值只在
        // subtree 內無牌組時作為 fallback（spec：資料夾修改日期 = 內含最新牌組的修改日期）。
        let (sql, with_pid): (&str, bool) = if parent_id.is_some() {
            (
                "WITH RECURSIVE subtree(root, descendant) AS (
                    SELECT id, id FROM decks
                    UNION ALL
                    SELECT s.root, c.id FROM subtree s JOIN decks c ON c.parent_id = s.descendant
                 )
                 SELECT d.id, d.name, d.color, COALESCE(w.cnt, 0), d.is_folder, d.parent_id,
                     CASE WHEN d.is_folder THEN
                         substr(COALESCE((SELECT MAX(c.updated_at) FROM decks c
                                   JOIN subtree s ON s.descendant = c.id
                                   WHERE s.root = d.id AND c.is_folder = 0),
                                  d.updated_at), 1, 16)
                     ELSE substr(d.updated_at, 1, 16) END AS updated_at
                 FROM decks d
                 LEFT JOIN (SELECT deck_id, COUNT(*) AS cnt FROM words GROUP BY deck_id) w
                   ON w.deck_id = d.id
                 WHERE d.parent_id = ?1
                 ORDER BY d.is_folder DESC, d.id",
                true,
            )
        } else {
            (
                "WITH RECURSIVE subtree(root, descendant) AS (
                    SELECT id, id FROM decks
                    UNION ALL
                    SELECT s.root, c.id FROM subtree s JOIN decks c ON c.parent_id = s.descendant
                 )
                 SELECT d.id, d.name, d.color, COALESCE(w.cnt, 0), d.is_folder, d.parent_id,
                     CASE WHEN d.is_folder THEN
                         substr(COALESCE((SELECT MAX(c.updated_at) FROM decks c
                                   JOIN subtree s ON s.descendant = c.id
                                   WHERE s.root = d.id AND c.is_folder = 0),
                                  d.updated_at), 1, 16)
                     ELSE substr(d.updated_at, 1, 16) END AS updated_at
                 FROM decks d
                 LEFT JOIN (SELECT deck_id, COUNT(*) AS cnt FROM words GROUP BY deck_id) w
                   ON w.deck_id = d.id
                 WHERE d.parent_id IS NULL
                 ORDER BY d.is_folder DESC, d.id",
                false,
            )
        };
        let mut stmt = conn.prepare(sql)?;
        let rows: Vec<Deck> = if with_pid {
            stmt.query_map(params![parent_id.unwrap()], |r| {
                Ok(Deck {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    color: r.get(2)?,
                    word_count: r.get(3)?,
                    is_folder: r.get::<_, i64>(4)? != 0,
                    parent_id: r.get(5)?,
                    updated_at: r.get(6)?,
                })
            })?.collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |r| {
                Ok(Deck {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    color: r.get(2)?,
                    word_count: r.get(3)?,
                    is_folder: r.get::<_, i64>(4)? != 0,
                    parent_id: r.get(5)?,
                    updated_at: r.get(6)?,
                })
            })?.collect::<Result<Vec<_>>>()?
        };
        Ok(rows)
    }

    pub fn get_folder_path(&self, folder_id: Option<i64>) -> Result<Vec<Deck>> {
        let mut path = Vec::new();
        let mut current = folder_id;
        let conn = self.conn.lock().unwrap();
        while let Some(fid) = current {
            let mut stmt = conn.prepare(
                "SELECT id, name, color, 0, is_folder, parent_id, substr(updated_at,1,16) FROM decks WHERE id = ?1",
            )?;
            let folder: Deck = stmt.query_row(params![fid], |r| {
                Ok(Deck {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    color: r.get(2)?,
                    word_count: 0,
                    is_folder: r.get::<_, i64>(4)? != 0,
                    parent_id: r.get(5)?,
                    updated_at: r.get(6)?,
                })
            })?;
            current = folder.parent_id;
            path.push(folder);
        }
        path.reverse();
        Ok(path)
    }

/// 直屬此層資料夾（parent）的非資料夾牌組，不含巢狀子資料夾內的牌組。
    pub fn list_direct_decks(&self, parent: Option<i64>) -> Result<Vec<Deck>> {
        let conn = self.conn.lock().unwrap();
        let (sql, with_pid): (&str, bool) = if parent.is_some() {
            (
                "SELECT d.id, d.name, d.color, COALESCE(w.cnt, 0), d.is_folder, d.parent_id, substr(d.updated_at, 1, 16)
                 FROM decks d
                 LEFT JOIN (SELECT deck_id, COUNT(*) AS cnt FROM words GROUP BY deck_id) w
                   ON w.deck_id = d.id
                 WHERE d.is_folder = 0 AND d.parent_id = ?1
                 ORDER BY d.name",
                true,
            )
        } else {
            (
                "SELECT d.id, d.name, d.color, COALESCE(w.cnt, 0), d.is_folder, d.parent_id, substr(d.updated_at, 1, 16)
                 FROM decks d
                 LEFT JOIN (SELECT deck_id, COUNT(*) AS cnt FROM words GROUP BY deck_id) w
                   ON w.deck_id = d.id
                 WHERE d.is_folder = 0 AND d.parent_id IS NULL
                 ORDER BY d.name",
                false,
            )
        };
        let mut stmt = conn.prepare(sql)?;
        let rows: Vec<Deck> = if with_pid {
            stmt.query_map(params![parent.unwrap()], |r| {
                Ok(Deck {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    color: r.get(2)?,
                    word_count: r.get(3)?,
                    is_folder: r.get::<_, i64>(4)? != 0,
                    parent_id: r.get(5)?,
                    updated_at: r.get(6)?,
                })
            })?.collect::<Result<Vec<_>>>()?
        } else {
            stmt.query_map([], |r| {
                Ok(Deck {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    color: r.get(2)?,
                    word_count: r.get(3)?,
                    is_folder: r.get::<_, i64>(4)? != 0,
                    parent_id: r.get(5)?,
                    updated_at: r.get(6)?,
                })
            })?.collect::<Result<Vec<_>>>()?
        };
        Ok(rows)
    }

    pub fn rename_deck(&self, id: i64, name: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE decks SET name = ?1, updated_at = datetime('now','localtime') WHERE id = ?2",
            params![name, id],
        )?;
        Ok(())
    }

    pub fn update_deck_color(&self, id: i64, color: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE decks SET color = ?1, updated_at = datetime('now','localtime') WHERE id = ?2",
            params![color, id],
        )?;
        Ok(())
    }

    pub fn add_word(&self, deck_id: i64, front: &str, back: &str,
        pos: &str, pron: &str, example: &str, synonym: &str, antonym: &str, tags: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO words (deck_id, front, back, pos, pron, example, synonym, antonym, tags) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![deck_id, front, back, pos, pron, example, synonym, antonym, tags],
        )?;
        conn.execute(
            "UPDATE decks SET updated_at = datetime('now','localtime') WHERE id = ?1",
            params![deck_id],
        )?;
        Ok(())
    }

    pub fn delete_deck(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // recursive delete: delete children first, then parent
        conn.execute("DELETE FROM decks WHERE parent_id = ?1", params![id])?;
        conn.execute("DELETE FROM decks WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Load id → (name, parent_id) for folders. When ids is empty, loads ALL folders.
    /// Used by search grouping to resolve parent folder paths without per-folder SQL.
    pub fn get_folder_map(&self, ids: &[i64]) -> Result<HashMap<i64, (String, Option<i64>)>> {
        let sql = if ids.is_empty() {
            "SELECT id, name, parent_id FROM decks WHERE is_folder = 1".to_string()
        } else {
            let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
            format!(
                "SELECT id, name, parent_id FROM decks WHERE id IN ({})",
                placeholders.join(",")
            )
        };
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::types::ToSql> =
            ids.iter().map(|id| id as &dyn rusqlite::types::ToSql).collect();
        let rows = stmt.query_map(params.as_slice(), |r| {
            Ok((r.get::<_, i64>(0)?, (r.get::<_, String>(1)?, r.get(2)?)))
        })?;
        let mut map = HashMap::new();
        for row in rows {
            let (id, (name, parent_id)) = row?;
            map.insert(id, (name, parent_id));
        }
        Ok(map)
    }

    pub fn search_decks(&self, query: &str) -> Result<Vec<Deck>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT d.id, d.name, d.color, COALESCE(w.cnt, 0), d.is_folder, d.parent_id, substr(COALESCE(d.updated_at, ''), 1, 16)
             FROM decks d
             LEFT JOIN (SELECT deck_id, COUNT(*) AS cnt FROM words GROUP BY deck_id) w ON w.deck_id = d.id
             WHERE d.name LIKE ?1
             ORDER BY d.is_folder DESC, d.name"
        )?;
        let rows = stmt.query_map(params![pattern], |r| {
            Ok(Deck {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
                word_count: r.get(3)?,
                is_folder: r.get::<_, i64>(4)? != 0,
                parent_id: r.get(5)?,
                updated_at: r.get(6)?,
            })
        })?.collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }
}
