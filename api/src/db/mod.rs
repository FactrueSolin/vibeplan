use std::path::PathBuf;

use anyhow::Context;
use plan_migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement};

pub async fn init_database(database_url: &str) -> anyhow::Result<DatabaseConnection> {
    ensure_sqlite_parent_dir(database_url).await?;

    let db = Database::connect(database_url).await?;
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "PRAGMA foreign_keys = ON;".to_owned(),
    ))
    .await?;
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        "PRAGMA journal_mode = WAL;".to_owned(),
    ))
    .await?;
    Migrator::up(&db, None).await?;

    Ok(db)
}

async fn ensure_sqlite_parent_dir(database_url: &str) -> anyhow::Result<()> {
    let Some(stripped) = database_url.strip_prefix("sqlite://") else {
        return Ok(());
    };

    let path = stripped.split('?').next().unwrap_or_default();
    if path.is_empty() || path == ":memory:" {
        return Ok(());
    }

    let file_path = PathBuf::from(path);
    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create database directory {}", parent.display()))?;
    }

    Ok(())
}
