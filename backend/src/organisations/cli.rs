//! The operator's commands: organisations, their settings, and the commands
//! that work in one organisation (user's decisions, 2026-11-12: organisations
//! are created by hand for early access, and their settings set here).

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use uuid::Uuid;

use crate::db::{Db, System, Tenants};
use crate::sessions::SessionGap;

const USAGE: &str = "\
usage:
  fleetranger serve
  fleetranger organisations
  fleetranger add-organisation <name> [--time-zone <zone>] [--session-gap <minutes>] [--mcp on|off]
  fleetranger set-organisation <id> [--name <name>] [--time-zone <zone>] [--session-gap <minutes>] [--mcp on|off]
  fleetranger add-user [--organisation <id>] [--admin] <email> <name>
  fleetranger import [--organisation <id>] [--group <name>]
--organisation may be left out while the install has one organisation.";

/// Runs a command other than `serve`.
pub async fn run(
    args: &[String],
    system: &System,
    tenants: &Tenants,
    log_dir: &Path,
) -> Result<()> {
    let Some((command, rest)) = args.split_first() else {
        bail!("{USAGE}");
    };
    let args = Args::parse(rest)?;
    match command.as_str() {
        "organisations" => {
            let [] = args.expect(&[])?;
            list(system, None).await
        }
        "add-organisation" => {
            let [name] = args.expect(&["time-zone", "session-gap", "mcp"])?;
            add(system, name, &args).await
        }
        "set-organisation" => {
            let [id] = args.expect(&["name", "time-zone", "session-gap", "mcp"])?;
            set(system, parse_id(id)?, &args).await
        }
        "add-user" => {
            let [email, name] = args.expect(&["organisation", "admin"])?;
            let mut db = organisation(system, tenants, &args).await?;
            crate::users::add(&mut db, email, name, args.flag("admin")).await
        }
        "import" => {
            let [] = args.expect(&["organisation", "group"])?;
            let mut db = organisation(system, tenants, &args).await?;
            crate::ingest::import::run(&mut db, log_dir, args.value("group")).await
        }
        other => bail!("unknown command {other:?}\n{USAGE}"),
    }
}

/// A command's words and `--name value` options; `--admin` takes no value.
struct Args<'a> {
    words: Vec<&'a str>,
    options: HashMap<&'a str, Option<&'a str>>,
}

impl<'a> Args<'a> {
    fn parse(args: &'a [String]) -> Result<Self> {
        let mut words = Vec::new();
        let mut options = HashMap::new();
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            match arg.strip_prefix("--") {
                Some("admin") => {
                    options.insert("admin", None);
                }
                Some(name) => {
                    let value = args
                        .next()
                        .with_context(|| format!("--{name} needs a value"))?;
                    if options.insert(name, Some(value.as_str())).is_some() {
                        bail!("--{name} is given twice");
                    }
                }
                None => words.push(arg.as_str()),
            }
        }
        Ok(Self { words, options })
    }

    /// The words, if there are this many and only these options.
    /// The `N` arguments, if there are exactly that many and only these options.
    fn expect<const N: usize>(&self, options: &[&str]) -> Result<[&'a str; N]> {
        let Ok(words) = <[&'a str; N]>::try_from(self.words.as_slice()) else {
            bail!("expected {N} arguments, got {}\n{USAGE}", self.words.len());
        };
        if let Some(unknown) = self.options.keys().find(|name| !options.contains(name)) {
            bail!("unknown option --{unknown}\n{USAGE}");
        }
        Ok(words)
    }

    fn value(&self, name: &str) -> Option<&'a str> {
        self.options.get(name).copied().flatten()
    }

    fn flag(&self, name: &str) -> bool {
        self.options.contains_key(name)
    }
}

fn parse_id(id: &str) -> Result<Uuid> {
    id.parse().with_context(|| {
        format!("{id:?} is not an organisation id; see `fleetranger organisations`")
    })
}

/// A connection in the organisation given with `--organisation`, or the only
/// one there is.
async fn organisation(system: &System, tenants: &Tenants, args: &Args<'_>) -> Result<Db> {
    let id = match args.value("organisation") {
        Some(id) => parse_id(id)?,
        None => {
            let ids: Vec<Uuid> = sqlx::query_scalar!("select id from organisations")
                .fetch_all(&**system)
                .await?;
            let [id] = ids[..] else {
                bail!("the install has several organisations: name one with --organisation <id>");
            };
            id
        }
    };
    let exists = sqlx::query_scalar!(
        r#"select exists(select 1 from organisations where id = $1) as "exists!""#,
        id
    )
    .fetch_one(&**system)
    .await?;
    if !exists {
        bail!("there is no organisation {id}; see `fleetranger organisations`");
    }
    Ok(tenants.connect(id).await?)
}

/// Prints the organisations with their settings, or only this one.
async fn list(system: &System, only: Option<Uuid>) -> Result<()> {
    let rows = sqlx::query!(
        "select o.id, o.name, s.time_zone, s.session_gap_minutes, s.mcp
         from organisations o join organisation_settings s on s.organisation_id = o.id
         where $1::uuid is null or o.id = $1
         order by o.created_at",
        only,
    )
    .fetch_all(&**system)
    .await?;
    for row in rows {
        println!(
            "{}  {}  time zone {}, session gap {} min, MCP {}",
            row.id,
            row.name,
            row.time_zone,
            row.session_gap_minutes,
            if row.mcp { "on" } else { "off" }
        );
    }
    Ok(())
}

/// The settings given, checked.
struct Changes {
    time_zone: Option<String>,
    session_gap: Option<SessionGap>,
    mcp: Option<bool>,
}

async fn changes(system: &System, args: &Args<'_>) -> Result<Changes> {
    let time_zone = match args.value("time-zone") {
        Some(zone) => {
            let known = sqlx::query_scalar!(
                r#"select exists(select 1 from pg_timezone_names where name = $1) as "known!""#,
                zone
            )
            .fetch_one(&**system)
            .await?;
            if !known {
                bail!("{zone:?} is not a known time zone, e.g. Europe/Amsterdam");
            }
            Some(zone.to_owned())
        }
        None => None,
    };
    let session_gap = match args.value("session-gap") {
        Some(minutes) => Some(SessionGap(
            minutes
                .parse()
                .ok()
                .filter(|minutes| *minutes > 0)
                .with_context(|| {
                    format!("--session-gap {minutes:?} is not a positive number of minutes")
                })?,
        )),
        None => None,
    };
    let mcp = match args.value("mcp") {
        Some("on") => Some(true),
        Some("off") => Some(false),
        Some(other) => bail!("--mcp takes on or off, not {other:?}"),
        None => None,
    };
    Ok(Changes {
        time_zone,
        session_gap,
        mcp,
    })
}

fn name(name: &str) -> Result<&str> {
    let name = name.trim();
    if name.is_empty() {
        bail!("the name is empty");
    }
    Ok(name)
}

/// A new organisation, with its settings and its first group, "General", as
/// every organisation has (migration 0025). Its first admin follows with
/// `add-user --organisation <id> --admin`.
async fn add(system: &System, given_name: &str, args: &Args<'_>) -> Result<()> {
    let name = name(given_name)?;
    let changes = changes(system, args).await?;
    let same_name = sqlx::query_scalar!(
        r#"select count(*) as "count!" from organisations where name = $1"#,
        name
    )
    .fetch_one(&**system)
    .await?;
    let mut tx = system.begin().await?;
    let id = sqlx::query_scalar!(
        "insert into organisations (name) values ($1) returning id",
        name
    )
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query!(
        "insert into organisation_settings (organisation_id, time_zone, session_gap_minutes, mcp)
         values ($1, coalesce($2, 'UTC'), coalesce($3, 120), coalesce($4, false))",
        id,
        changes.time_zone,
        changes.session_gap.map(|SessionGap(minutes)| minutes),
        changes.mcp,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "insert into groups (organisation_id, name) values ($1, 'General')",
        id
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    if same_name > 0 {
        println!("note: another organisation is also called {name:?}");
    }
    println!("{id}  {name}");
    Ok(())
}

async fn set(system: &System, id: Uuid, args: &Args<'_>) -> Result<()> {
    let name = args.value("name").map(name).transpose()?;
    let changes = changes(system, args).await?;
    let mut tx = system.begin().await?;
    let found = sqlx::query!(
        "update organisations set name = coalesce($2, name) where id = $1",
        id,
        name,
    )
    .execute(&mut *tx)
    .await?;
    if found.rows_affected() == 0 {
        bail!("there is no organisation {id}; see `fleetranger organisations`");
    }
    sqlx::query!(
        "update organisation_settings
         set time_zone = coalesce($2, time_zone),
             session_gap_minutes = coalesce($3, session_gap_minutes),
             mcp = coalesce($4, mcp)
         where organisation_id = $1",
        id,
        changes.time_zone,
        changes.session_gap.map(|SessionGap(minutes)| minutes),
        changes.mcp,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    list(system, Some(id)).await
}
