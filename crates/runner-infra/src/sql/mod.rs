use rusqlite::Connection;

pub async fn execute_batch(
    dsn: &str,
    query: Option<&str>,
    sql_file: Option<&str>,
) -> Result<(), String> {
    let dsn = dsn.to_string();
    let query = query.map(ToString::to_string);
    let sql_file = sql_file.map(ToString::to_string);

    tokio::task::spawn_blocking(move || {
        let path = normalize_sqlite_dsn(&dsn)?;
        let conn = Connection::open(path).map_err(|e| format!("open sqlite failed: {}", e))?;

        let script = if let Some(q) = query {
            q
        } else if let Some(file) = sql_file {
            std::fs::read_to_string(&file)
                .map_err(|e| format!("read sql file failed '{}': {}", file, e))?
        } else {
            return Err("sql task requires query or sql_file".to_string());
        };

        conn.execute_batch(&script)
            .map_err(|e| format!("execute sql batch failed: {}", e))
    })
    .await
    .map_err(|e| format!("sql task join error: {}", e))?
}

fn normalize_sqlite_dsn(dsn: &str) -> Result<String, String> {
    let value = dsn.trim();
    if value == "sqlite::memory:" {
        return Ok(":memory:".to_string());
    }
    if let Some(path) = value.strip_prefix("sqlite://") {
        if path.is_empty() {
            return Err("invalid sqlite dsn: missing path".to_string());
        }
        return Ok(path.to_string());
    }
    Err(format!(
        "unsupported sql dsn '{}': use sqlite://<path> or sqlite::memory:",
        value
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn execute_batch_memory_success() {
        let sql = "CREATE TABLE IF NOT EXISTS users(id INTEGER); INSERT INTO users(id) VALUES (1);";
        let res = execute_batch("sqlite::memory:", Some(sql), None).await;
        assert!(res.is_ok());
    }
}
