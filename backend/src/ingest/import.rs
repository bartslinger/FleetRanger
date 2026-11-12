//! Registers raw logs that are already in the flight log directory, such as logs
//! copied there before FleetRanger existed. Files stay where they are. Their
//! flights go to the group given, which is needed only when there is more
//! than one (user's decision, 2026-11-11): nothing falls back to a default.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use super::db;

pub async fn run(db: &PgPool, log_dir: &Path, group: Option<&str>) -> Result<()> {
    let group_id = choose_group(&db::groups(db).await?, group)?;
    let mut paths = Vec::new();
    collect_logs(log_dir, &mut paths)
        .with_context(|| format!("failed to scan {}", log_dir.display()))?;
    paths.sort();

    let known = db::known_paths(db).await?;
    let (mut added, mut duplicates) = (0, 0);
    for path in paths {
        let relative = relative_path(log_dir, &path)?;
        if known.contains(&relative) {
            continue;
        }

        let bytes = tokio::fs::read(&path)
            .await
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sha256 = Sha256::digest(&bytes);
        if db::insert_raw_log(db, &sha256, &relative, bytes.len() as i64, group_id).await? {
            added += 1;
        } else {
            duplicates += 1;
            tracing::info!(path = relative, "skipped: same content as a registered log");
        }
    }

    tracing::info!(added, duplicates, "import finished");
    Ok(())
}

/// The group named, or the only one when none is named.
fn choose_group(groups: &[(Uuid, String)], name: Option<&str>) -> Result<Uuid> {
    match (name, groups) {
        (Some(name), _) => match groups.iter().find(|(_, group)| group == name) {
            Some((id, _)) => Ok(*id),
            None => bail!("there is no group named {name:?}"),
        },
        (None, [(id, _)]) => Ok(*id),
        (None, _) => bail!("there are several groups; choose one with --group <name>"),
    }
}

/// Collects every `.txt` file below `dir`.
fn collect_logs(dir: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();
        if file_type.is_dir() {
            collect_logs(&path, paths)?;
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "txt") {
            paths.push(path);
        }
    }
    Ok(())
}

fn relative_path(log_dir: &Path, path: &Path) -> Result<String> {
    let relative = path.strip_prefix(log_dir)?;
    match relative.to_str() {
        Some(relative) => Ok(relative.to_owned()),
        None => bail!("file name is not valid UTF-8: {}", path.display()),
    }
}
