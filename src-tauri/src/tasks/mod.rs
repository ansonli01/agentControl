use std::sync::Mutex;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::state::data_dir;

/// 历史任务记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRecord {
    pub id: String,
    /// 毫秒时间戳
    pub ts: i64,
    /// detect / install / upgrade / uninstall
    pub kind: String,
    pub agent_id: String,
    pub agent_name: String,
    pub target: String,
    pub source: String,
    /// success / failed / cancelled
    pub result: String,
    pub duration_ms: i64,
    pub message: Option<String>,
}

/// SQLite 历史任务存储。
pub struct TaskDb {
    conn: Mutex<Connection>,
}
impl TaskDb {
    /// 打开数据库并建表。
    pub fn open() -> AppResult<Self> {
        let path = data_dir()?.join("history.db");
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                ts INTEGER NOT NULL,
                kind TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                agent_name TEXT NOT NULL,
                target TEXT NOT NULL,
                source TEXT NOT NULL,
                result TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                message TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_tasks_ts ON tasks(ts DESC);",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// 写入一条记录。
    pub fn insert(&self, rec: &TaskRecord) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO tasks
             (id, ts, kind, agent_id, agent_name, target, source, result, duration_ms, message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                rec.id,
                rec.ts,
                rec.kind,
                rec.agent_id,
                rec.agent_name,
                rec.target,
                rec.source,
                rec.result,
                rec.duration_ms,
                rec.message
            ],
        )?;
        Ok(())
    }

    /// 查询历史任务（按时间倒序）。
    pub fn list(&self, limit: i64, kind: Option<String>) -> AppResult<Vec<TaskRecord>> {
        let conn = self.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, ts, kind, agent_id, agent_name, target, source, result, duration_ms, message
             FROM tasks WHERE (?1 IS NULL OR kind = ?1) ORDER BY ts DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![kind, limit], |row| {
            Ok(TaskRecord {
                id: row.get(0)?,
                ts: row.get(1)?,
                kind: row.get(2)?,
                agent_id: row.get(3)?,
                agent_name: row.get(4)?,
                target: row.get(5)?,
                source: row.get(6)?,
                result: row.get(7)?,
                duration_ms: row.get(8)?,
                message: row.get(9)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// 清空历史。
    pub fn clear(&self) -> AppResult<()> {
        let conn = self.conn.lock().map_err(|e| crate::error::AppError::msg(e.to_string()))?;
        conn.execute("DELETE FROM tasks", [])?;
        Ok(())
    }
}