//! Database layer using rusqlite
//!
//! Handles all SQLite operations including migrations, CRUD for monitors,
//! check results, and agent data.

use rusqlite::{Connection, params};
use std::sync::Mutex;

use crate::models::*;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Create a new database connection
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Run database migrations to create tables
    pub fn run_migrations(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS monitors (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                monitor_type TEXT NOT NULL,
                target TEXT NOT NULL,
                port INTEGER,
                interval_seconds INTEGER NOT NULL DEFAULT 60,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS check_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                monitor_id TEXT NOT NULL,
                status TEXT NOT NULL,
                response_time_ms REAL,
                status_code INTEGER,
                message TEXT,
                checked_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (monitor_id) REFERENCES monitors(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS ssl_certificates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                monitor_id TEXT NOT NULL,
                subject TEXT,
                issuer TEXT,
                not_before TEXT,
                not_after TEXT,
                days_until_expiry INTEGER,
                checked_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (monitor_id) REFERENCES monitors(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                hostname TEXT NOT NULL,
                os TEXT,
                kernel TEXT,
                ip_address TEXT,
                last_seen TEXT NOT NULL DEFAULT (datetime('now')),
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS agent_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id TEXT NOT NULL,
                cpu_usage REAL,
                ram_total_mb REAL,
                ram_used_mb REAL,
                ram_cached_mb REAL,
                ram_available_mb REAL,
                uptime_seconds INTEGER,
                reported_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS agent_disks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                metric_id INTEGER NOT NULL,
                mount_point TEXT NOT NULL,
                total_gb REAL,
                used_gb REAL,
                available_gb REAL,
                usage_percent REAL,
                FOREIGN KEY (metric_id) REFERENCES agent_metrics(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS agent_network (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                metric_id INTEGER NOT NULL,
                interface TEXT NOT NULL,
                rx_bytes INTEGER,
                tx_bytes INTEGER,
                rx_rate_bps REAL,
                tx_rate_bps REAL,
                FOREIGN KEY (metric_id) REFERENCES agent_metrics(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS agent_processes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                metric_id INTEGER NOT NULL,
                pid INTEGER,
                name TEXT,
                cpu_percent REAL,
                memory_mb REAL,
                FOREIGN KEY (metric_id) REFERENCES agent_metrics(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_check_results_monitor ON check_results(monitor_id, checked_at);
            CREATE INDEX IF NOT EXISTS idx_agent_metrics_agent ON agent_metrics(agent_id, reported_at);
            "
        )?;
        Ok(())
    }

    // ─── Monitor CRUD ───────────────────────────────────────────────────────

    /// Insert a new monitor
    pub fn insert_monitor(&self, monitor: &Monitor) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO monitors (id, name, monitor_type, target, port, interval_seconds, enabled)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                monitor.id,
                monitor.name,
                monitor.monitor_type.as_str(),
                monitor.target,
                monitor.port,
                monitor.interval_seconds,
                monitor.enabled as i32,
            ],
        )?;
        Ok(())
    }

    /// Get all monitors
    pub fn get_monitors(&self) -> Result<Vec<Monitor>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, monitor_type, target, port, interval_seconds, enabled FROM monitors"
        )?;
        let monitors = stmt.query_map([], |row| {
            Ok(Monitor {
                id: row.get(0)?,
                name: row.get(1)?,
                monitor_type: MonitorType::from_str(&row.get::<_, String>(2)?),
                target: row.get(3)?,
                port: row.get(4)?,
                interval_seconds: row.get(5)?,
                enabled: row.get::<_, i32>(6)? != 0,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(monitors)
    }

    /// Delete a monitor by ID
    pub fn delete_monitor(&self, id: &str) -> Result<bool, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute("DELETE FROM monitors WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    // ─── Check Results ──────────────────────────────────────────────────────

    /// Insert a check result
    pub fn insert_check_result(&self, result: &CheckResult) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO check_results (monitor_id, status, response_time_ms, status_code, message)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                result.monitor_id,
                result.status.as_str(),
                result.response_time_ms,
                result.status_code,
                result.message,
            ],
        )?;
        Ok(())
    }

    /// Get recent check results for a monitor
    pub fn get_check_results(&self, monitor_id: &str, limit: u32) -> Result<Vec<CheckResult>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT monitor_id, status, response_time_ms, status_code, message, checked_at
             FROM check_results WHERE monitor_id = ?1 ORDER BY checked_at DESC LIMIT ?2"
        )?;
        let results = stmt.query_map(params![monitor_id, limit], |row| {
            Ok(CheckResult {
                monitor_id: row.get(0)?,
                status: CheckStatus::from_str(&row.get::<_, String>(1)?),
                response_time_ms: row.get(2)?,
                status_code: row.get(3)?,
                message: row.get(4)?,
                checked_at: row.get(5)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(results)
    }

    // ─── Agent Data ─────────────────────────────────────────────────────────

    /// Upsert an agent registration
    pub fn upsert_agent(&self, agent: &AgentInfo) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO agents (id, hostname, os, kernel, ip_address, last_seen)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))
             ON CONFLICT(id) DO UPDATE SET
                hostname=excluded.hostname, os=excluded.os, kernel=excluded.kernel,
                ip_address=excluded.ip_address, last_seen=datetime('now')",
            params![agent.id, agent.hostname, agent.os, agent.kernel, agent.ip_address],
        )?;
        Ok(())
    }

    /// Insert agent metrics and return the metric ID
    pub fn insert_agent_metrics(&self, metrics: &AgentMetrics) -> Result<i64, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO agent_metrics (agent_id, cpu_usage, ram_total_mb, ram_used_mb, ram_cached_mb, ram_available_mb, uptime_seconds)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                metrics.agent_id,
                metrics.cpu_usage,
                metrics.ram_total_mb,
                metrics.ram_used_mb,
                metrics.ram_cached_mb,
                metrics.ram_available_mb,
                metrics.uptime_seconds,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Get all agents
    pub fn get_agents(&self) -> Result<Vec<AgentInfo>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, hostname, os, kernel, ip_address FROM agents ORDER BY last_seen DESC"
        )?;
        let agents = stmt.query_map([], |row| {
            Ok(AgentInfo {
                id: row.get(0)?,
                hostname: row.get(1)?,
                os: row.get(2)?,
                kernel: row.get(3)?,
                ip_address: row.get(4)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(agents)
    }
}
