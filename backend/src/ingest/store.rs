//! Where raw logs are kept: files in the log directory (`FLEETRANGER_LOG_DIR`),
//! or objects in an S3 bucket when `FLEETRANGER_S3_BUCKET` is set (user's
//! decisions, 2026-11-12 and 2026-11-13). Self-hosters keep them on disk; the
//! hosted service uses S3. Both name a log the same way: its organisation's
//! id, then the path stored with it, such as
//! `<organisation id>/raw/<controller>/FlightRecord/<file>`.
//!
//! Logs come in through scratch files in the log directory: an upload is
//! written to one while its SHA-256 is computed, and stored only once it
//! checks out. To process a log from S3, it is fetched into one, as the
//! parser's child process reads a file and has no network.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};
use futures_util::StreamExt;
use object_store::ObjectStoreExt;
use object_store::aws::{AmazonS3, AmazonS3Builder, AmazonS3ConfigKey};
use object_store::buffered::BufWriter;
use object_store::path::Path as ObjectPath;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

/// The scratch files' directory, in the log directory. Not in an
/// organisation's directory, so the import never takes them for logs.
const SCRATCH_DIR: &str = ".scratch";
/// A scratch file this old was left by a backend that stopped. Younger
/// ones may belong to another backend sharing the log directory.
const SCRATCH_MAX_AGE: Duration = Duration::from_secs(3600);

#[derive(Clone)]
pub struct LogStore {
    log_dir: PathBuf,
    /// The bucket, if logs are kept on S3.
    s3: Option<Arc<AmazonS3>>,
}

impl LogStore {
    /// Logs on disk, or in `s3_bucket`, reached with the usual `AWS_*`
    /// variables: credentials (or, on EC2, the instance's role), and for
    /// another provider than AWS, `AWS_ENDPOINT` and `AWS_REGION`.
    pub fn open(log_dir: PathBuf, s3_bucket: Option<&str>) -> Result<Self> {
        let s3 = match s3_bucket {
            Some(bucket) => {
                // As `AmazonS3Builder::from_env`, but leaving out empty
                // variables, as Compose passes those it has no value for.
                let mut builder = AmazonS3Builder::new();
                for (name, value) in std::env::vars() {
                    if name.starts_with("AWS_")
                        && !value.is_empty()
                        && let Ok(key) = name.to_ascii_lowercase().parse::<AmazonS3ConfigKey>()
                    {
                        builder = builder.with_config(key, value);
                    }
                }
                Some(Arc::new(
                    builder
                        .with_bucket_name(bucket)
                        .build()
                        .context("failed to set up the S3 bucket")?,
                ))
            }
            None => None,
        };
        Ok(Self { log_dir, s3 })
    }

    /// Where the logs are, for the server log.
    pub fn describe(&self) -> &'static str {
        if self.s3.is_some() { "S3" } else { "disk" }
    }

    /// An empty scratch file, removed again when dropped unless stored.
    pub async fn scratch(&self) -> Result<Scratch> {
        let dir = self.log_dir.join(SCRATCH_DIR);
        tokio::fs::create_dir_all(&dir)
            .await
            .with_context(|| format!("failed to create {}", dir.display()))?;
        let path = dir.join(Uuid::new_v4().to_string());
        let file = tokio::fs::File::create_new(&path)
            .await
            .with_context(|| format!("failed to create {}", path.display()))?;
        Ok(Scratch {
            path,
            file: Some(file),
            keep: false,
        })
    }

    /// Removes scratch files that backends left when they stopped.
    pub fn remove_old_scratch(&self) {
        let Ok(entries) = std::fs::read_dir(self.log_dir.join(SCRATCH_DIR)) else {
            return;
        };
        for entry in entries.flatten() {
            let old = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .ok()
                .and_then(|modified| SystemTime::now().duration_since(modified).ok())
                .is_some_and(|age| age > SCRATCH_MAX_AGE);
            if old && let Err(error) = std::fs::remove_file(entry.path()) {
                tracing::warn!(path = %entry.path().display(), "failed to remove a scratch file: {error}");
            }
        }
    }

    /// Stores a scratch file as an organisation's log at `path`, replacing
    /// any earlier version. On disk it is moved into place in one step, so
    /// the processing loop never reads half a log.
    pub async fn store(&self, organisation_id: Uuid, path: &str, scratch: Scratch) -> Result<()> {
        let mut scratch = scratch;
        // Closed first: all written, and nothing holds it while it moves.
        if let Some(mut file) = scratch.file.take() {
            file.flush().await?;
        }
        match &self.s3 {
            Some(bucket) => put(bucket, &object_path(organisation_id, path)?, &scratch.path).await,
            None => {
                let destination = self.file(organisation_id, path);
                if let Some(dir) = destination.parent() {
                    tokio::fs::create_dir_all(dir)
                        .await
                        .with_context(|| format!("failed to create {}", dir.display()))?;
                }
                tokio::fs::rename(&scratch.path, &destination)
                    .await
                    .with_context(|| format!("failed to store {path}"))?;
                scratch.keep = true;
                Ok(())
            }
        }
    }

    /// Copies a file in the organisation's log directory to S3, for the
    /// import; on disk it is where it belongs already.
    pub async fn store_in_place(&self, organisation_id: Uuid, path: &str) -> Result<()> {
        match &self.s3 {
            Some(bucket) => {
                put(
                    bucket,
                    &object_path(organisation_id, path)?,
                    &self.file(organisation_id, path),
                )
                .await
            }
            None => Ok(()),
        }
    }

    /// A local file with an organisation's log at `path`: the log itself on
    /// disk, or a scratch file with a copy from S3.
    pub async fn fetch(&self, organisation_id: Uuid, path: &str) -> Result<LocalLog> {
        let Some(bucket) = &self.s3 else {
            return Ok(LocalLog::File(self.file(organisation_id, path)));
        };
        let mut scratch = self.scratch().await?;
        let mut stream = bucket
            .get(&object_path(organisation_id, path)?)
            .await
            .with_context(|| format!("failed to fetch {path} from S3"))?
            .into_stream();
        let file = scratch.file()?;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.with_context(|| format!("failed to fetch {path} from S3"))?;
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
        Ok(LocalLog::Scratch(scratch))
    }

    /// The organisation's log directory, for the import.
    pub fn organisation_dir(&self, organisation_id: Uuid) -> PathBuf {
        self.log_dir.join(organisation_id.to_string())
    }

    fn file(&self, organisation_id: Uuid, path: &str) -> PathBuf {
        self.organisation_dir(organisation_id).join(path)
    }
}

/// The object's name, as it is: DJI's file names have brackets, which
/// `ObjectPath::from` would percent-encode.
fn object_path(organisation_id: Uuid, path: &str) -> Result<ObjectPath> {
    ObjectPath::parse(format!("{organisation_id}/{path}"))
        .with_context(|| format!("{path} can't be the name of an object"))
}

/// Uploads a local file, in parts if it is large, without holding it whole.
async fn put(bucket: &Arc<AmazonS3>, object: &ObjectPath, file: &Path) -> Result<()> {
    let mut source = tokio::fs::File::open(file)
        .await
        .with_context(|| format!("failed to open {}", file.display()))?;
    let mut writer = BufWriter::new(bucket.clone(), object.clone());
    let copied = tokio::io::copy(&mut source, &mut writer).await;
    if let Err(error) = copied {
        // Leaves no unfinished parts behind.
        if let Err(abort) = writer.abort().await {
            tracing::warn!("failed to abort an upload to S3: {abort}");
        }
        return Err(error).with_context(|| format!("failed to store {object} on S3"));
    }
    writer
        .shutdown()
        .await
        .with_context(|| format!("failed to store {object} on S3"))
}

/// A file in the scratch directory, removed when dropped unless stored.
pub struct Scratch {
    path: PathBuf,
    file: Option<tokio::fs::File>,
    keep: bool,
}

impl Scratch {
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The file, for writing.
    pub fn file(&mut self) -> Result<&mut tokio::fs::File> {
        self.file.as_mut().context("the scratch file is closed")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if !self.keep
            && let Err(error) = std::fs::remove_file(&self.path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(path = %self.path.display(), "failed to remove a scratch file: {error}");
        }
    }
}

/// A log as a local file, for the parser.
pub enum LocalLog {
    File(PathBuf),
    Scratch(Scratch),
}

impl LocalLog {
    pub fn path(&self) -> &Path {
        match self {
            LocalLog::File(path) => path,
            LocalLog::Scratch(scratch) => scratch.path(),
        }
    }
}

/// The SHA-256 and size of a file, read in parts.
pub async fn hash_file(path: &Path) -> Result<(Vec<u8>, u64)> {
    let mut file = tokio::fs::File::open(path)
        .await
        .with_context(|| format!("failed to open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut size = 0;
    let mut buffer = vec![0; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .await
            .with_context(|| format!("failed to read {}", path.display()))?;
        let Some(chunk) = buffer.get(..read).filter(|chunk| !chunk.is_empty()) else {
            break;
        };
        hasher.update(chunk);
        size += read as u64;
    }
    Ok((hasher.finalize().to_vec(), size))
}
