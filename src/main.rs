use anyhow::bail;
use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use config::Config;
use hex::encode as hex_encode;
use hmac::Hmac;
use icon::make_icon;
use models::ImportData;
use rusqlite::Connection;
use serde_json::to_string_pretty;
use sha1::Digest;
use sha1::Sha1;

pub type HmacSha1 = Hmac<Sha1>;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;
// if you’re on sha1 ≥0.9 you can keep `use digest::Digest;`
use std::{
    fs,
    io::{Write, stdout},
};

mod config;
mod db;
mod gtf_texture;
mod icon;
mod labels;
mod models;
mod resource_dl;
mod resource_parse;
mod serializers;
mod xxtea;

use crate::resource_dl::{DownloadResult, download_level};
use db::{
    GameVersion, LevelType, SlotInfo, fetch_all_assets, fetch_all_levels, fetch_all_relations,
    fetch_all_users, get_slot_info,
};
use resource_parse::{ResrcData, ResrcDescriptor, ResrcMethod};
use serializers::lbp::{make_savearchive, make_slotlist};
use serializers::ps3::{make_pfd, make_sfo};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Download level and save as level backup
    Bkp {
        /// Level ID from database
        level_id: i64,
        /// Force LBP3 backup
        #[arg(short, long)]
        lbp3: bool,
    },

    Planet {
        /// 40‐hex SHA1 of the planet rootLevel
        hash: String,
    },

    // FetchPlanet {
    //     /// 40-hex SHA1 of the planet rootLevel
    //     hash: String,
    // },
    FetchLevel {
        /// Numeric level ID from database
        level_id: i64,
    },
    FetchEntirePlanet {
        /// npHandle of the user whose entire “planet” you want
        np_handle: String,
    },

    #[command(name = "read-from-file")]
    ReadFromFile,
}

async fn dl_as_planet(hash: &str, config: &Config) -> Result<()> {
    // 1) parse hex → [u8;20]
    let raw = hex::decode(hash).map_err(|e| anyhow!("invalid hex for hash: {}", e))?;
    if raw.len() != 20 {
        bail!("hash must be 20 bytes (40 hex chars)");
    }
    let mut root_hash = [0u8; 20];
    root_hash.copy_from_slice(&raw);

    // 2) grab all resources
    let DownloadResult {
        resources,
        success_count,
        error_count,
    } = download_level(
        root_hash,
        /* icon_sha1 = */ None,
        config.archive_path.to_string_lossy().into_owned(),
        config.max_parallel_downloads,
    )
    .await?;

    println!(
        "Done fetching {} resources ({}/{})",
        root_hash.iter().count(),
        success_count,
        error_count
    );

    // 3) inspect root to discover revision & game version
    let root_data = resources
        .get(&root_hash)
        .ok_or_else(|| anyhow!("rootLevel missing from archive"))?;
    let root_resrc = ResrcData::new(root_data, false)?;
    let revision = match root_resrc.method {
        ResrcMethod::Binary { revision, .. } => revision,
        _ => bail!("rootLevel is not a Binary resource"),
    };
    let gameversion = revision.get_gameversion();

    // 4) choose backup folder name
    let hash_up = hash.to_uppercase();
    // e.g. Backups/BCES01663PLANET3622E8...
    let bkp_name = format!("{}PLANET{}", gameversion.get_titleid(), hash_up);
    let bkp_path = config.backup_directory.join(&bkp_name);
    fs::create_dir_all(&bkp_path)?;

    // 5) build a dummy SlotInfo for a planet
    let slot_info = SlotInfo {
        name: format!("Planet {}", hash_up),
        description: String::new(),
        np_handle: String::new(),
        root_level: root_hash,
        icon: ResrcDescriptor::Guid(0), // no icon
        game: gameversion,
        initially_locked: false,
        is_sub_level: false,
        background_guid: None,
        shareable: false,
        author_labels: Vec::new(),
        leveltype: LevelType::Cooperative,
        min_players: None,
        max_players: None,
        is_adventure_planet: true,
    };

    // 6) slotlist
    let slt = make_slotlist(&revision, &slot_info)?;
    let slt_hash: [u8; 20] = {
        let mut h = Sha1::new();
        h.update(&slt);
        h.finalize().into()
    };

    // 7) write ICON0.PNG (none) and archive chunks
    let mut all_resources = resources;
    all_resources.insert(slt_hash, slt.clone());
    make_icon(&bkp_path, None, &mut all_resources)?;
    make_savearchive(&revision, slt_hash, all_resources, &bkp_path)?;

    // 8) PARAM.SFO + PARAM.PFD
    let sfo = make_sfo(&slot_info, &bkp_name, &bkp_path, &gameversion)?;
    let pfd_version = if gameversion == GameVersion::Lbp3 {
        4
    } else {
        3
    };
    make_pfd(pfd_version, sfo, &bkp_path)?;

    println!("Backup written to {}", bkp_path.display());
    Ok(())
}

async fn dl_as_backup(level_id: i64, config: Config, force_lbp3: bool) -> Result<()> {
    let slot_info = get_slot_info(level_id, &config.database_path)?;

    println!("Level found!");
    println!("  Name:    {}", &slot_info.name);
    println!("  Creator: {}", &slot_info.np_handle);
    println!("  Game:    {}", slot_info.game.get_short_title());

    // clamp parallelism
    let mut max_parallel = config.max_parallel_downloads;
    if max_parallel > 10 {
        eprintln!("WARNING: max_parallel_downloads is too high, reverting to 10");
        max_parallel = 10;
    } else if max_parallel == 0 {
        return Err(anyhow!("max_parallel_downloads cannot be zero"));
    }

    print!("Gathering resources from local archive…");
    stdout().flush()?;

    // extract icon hash if present
    let icon_sha1 = match slot_info.icon {
        ResrcDescriptor::Sha1(h) => Some(h),
        _ => None,
    };

    // call your local-archive-backed downloader
    let DownloadResult {
        resources: mut resources,
        success_count: dl_count,
        error_count: fail_count,
    } = download_level(
        slot_info.root_level,
        icon_sha1,
        config.archive_path.to_string_lossy().into_owned(), // your local archive root
        max_parallel,
    )
    .await?;

    println!("\nDone!  {dl_count} fetched, {fail_count} missing.");

    use crate::resource_parse::{ResrcData, ResrcMethod};
    use std::fs::OpenOptions;
    let mut dbg = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("output.txt")?;
    writeln!(dbg, "parent_sha1 <- dependency_sha1")?;
    for (parent_sha, blob) in &resources {
        if let Ok(res) = ResrcData::new(blob, false) {
            if let ResrcMethod::Binary { dependencies, .. } = res.method {
                for dep in dependencies {
                    if let ResrcDescriptor::Sha1(child_sha) = dep.desc {
                        writeln!(
                            dbg,
                            "{} <- {}",
                            hex_encode(parent_sha),
                            hex_encode(&child_sha)
                        )?;
                    }
                }
            }
        }
    }

    // pull out the root-level resource for version inspection
    let root_data = resources
        .get(&slot_info.root_level)
        .ok_or_else(|| anyhow!("rootLevel is missing from the archive"))?;
    let root_resrc = ResrcData::new(root_data, false)?;
    let mut revision = match root_resrc.method {
        ResrcMethod::Binary { revision, .. } => revision,
        _ => return Err(anyhow!("rootLevel uses non-binary serialization method")),
    };

    // optionally force to LBP3 revision, or warn/fix mismatches
    let mut gameversion = revision.get_gameversion();
    if force_lbp3 && gameversion != GameVersion::Lbp3 {
        eprintln!("WARNING: forcing LBP3 backup format");
        gameversion = GameVersion::Lbp3;
        revision = gameversion.get_latest_revision();
    } else if slot_info.game != gameversion {
        eprintln!(
            "WARNING: this is a {} level in {} format",
            slot_info.game.get_short_title(),
            gameversion.get_short_title(),
        );
        if config.fix_backup_version {
            eprintln!(
                "WARNING: writing backup as {}",
                gameversion.get_short_title()
            );
        } else {
            eprintln!(
                "WARNING: writing as {}, you may need to backport this level",
                slot_info.game.get_short_title()
            );
            gameversion = slot_info.game;
            revision = gameversion.get_latest_revision();
        }
    }

    // prepare output folder
    let slot_id_str = hex::encode_upper(u32::to_be_bytes(level_id as u32));
    let bkp_name = if slot_info.is_adventure_planet {
        format!("{}ADVLBP3AAZ{}", gameversion.get_titleid(), slot_id_str)
    } else {
        format!("{}LEVEL{}", gameversion.get_titleid(), slot_id_str)
    };
    let bkp_path = config.backup_directory.join(&bkp_name);
    fs::create_dir_all(&bkp_path)?;

    // build and insert the slotlist resource
    let slt = make_slotlist(&revision, &slot_info)?;

    // hash into [u8;20]
    let slt_hash: [u8; 20] = {
        let mut hasher = Sha1::new();
        hasher.update(&slt);
        // if sha1 ≥ 0.9:
        hasher.finalize().into()
        // if sha1 ≤ 0.8:
        // let d = hasher.digest();
        // d.bytes()
    };

    resources.insert(slt_hash, slt);

    // generate ICON0.PNG
    make_icon(&bkp_path, icon_sha1, &mut resources)?;

    // write the save-archive chunks
    make_savearchive(&revision, slt_hash, resources, &bkp_path)?;

    // write PARAM.SFO and PARAM.PFD
    let sfo = make_sfo(&slot_info, &bkp_name, &bkp_path, &gameversion)?;
    let pfd_version = if gameversion == GameVersion::Lbp3 {
        4
    } else {
        3
    };
    make_pfd(pfd_version, sfo, &bkp_path)?;

    println!("Backup written to {}", bkp_path.display());
    Ok(())
}

async fn fetch_planet_resources(hash: &str, config: &Config) -> Result<()> {
    // 1) hex → [u8;20]
    let raw = hex::decode(hash)?;
    if raw.len() != 20 {
        bail!("hash must be 20 bytes (40 hex chars)");
    }
    let mut planet_hash = [0u8; 20];
    planet_hash.copy_from_slice(&raw);

    // 2) download the SLTb blob (no icon)
    let DownloadResult {
        mut resources,
        success_count,
        error_count,
    } = download_level(
        planet_hash,
        None,
        config.archive_path.to_string_lossy().into_owned(),
        config.max_parallel_downloads,
    )
    .await?;

    // 3) parse the SLTb to extract each level’s root hash
    let slt_buf = resources
        .get(&planet_hash)
        .ok_or_else(|| anyhow!("planet SLTb missing"))?;
    let slt_meta = ResrcData::new(slt_buf, false)?;
    let mut level_hashes = Vec::new();
    if let ResrcMethod::Binary { dependencies, .. } = slt_meta.method {
        for dep in dependencies {
            if let ResrcDescriptor::Sha1(h) = dep.desc {
                level_hashes.push(h);
            }
        }
    }

    // 4) for each level, pull _all_ of its blobs
    for level_hash in level_hashes {
        let DownloadResult {
            resources: lvl_res, ..
        } = download_level(
            level_hash,
            None,
            config.archive_path.to_string_lossy().into_owned(),
            config.max_parallel_downloads,
        )
        .await?;
        for (sha, blob) in lvl_res {
            resources.insert(sha, blob);
        }
        println!("  → added level {}", hex::encode(level_hash));
    }

    // 5) write them all out as <hex>.bin
    let out_dir = config
        .backup_directory
        .join(format!("planet_{}", hash.to_uppercase()));
    fs::create_dir_all(&out_dir)?;
    for (sha, data) in &resources {
        let fname = format!("{}", hex::encode(sha));
        fs::write(out_dir.join(&fname), data)?;
    }
    println!("wrote {} files to {}", success_count, out_dir.display());

    // 6) write the planet root‐hash itself
    let planet_hex = hex::encode(planet_hash);
    fs::write(out_dir.join("planet_hash.txt"), &planet_hex)?;
    println!("wrote planet_hash.txt → {}", planet_hex);

    // 7) lookup & write the creator’s icon SHA1
    let conn = Connection::open(&config.database_path)?;
    let icon_blob: Vec<u8> = conn.query_row(
        // find slot row whose rootLevel equals our planet hash
        "SELECT u.icon
           FROM slot AS s
           JOIN \"user\" AS u ON s.npHandle = u.npHandle
          WHERE s.rootLevel = ?1",
        [&planet_hash],
        |r| r.get(0),
    )?;
    let icon_hex = hex::encode(&icon_blob);
    fs::write(out_dir.join("creator_icon_hash.txt"), &icon_hex)?;
    println!("wrote creator_icon_hash.txt → {}", icon_hex);

    Ok(())
}

async fn fetch_planet_resources_helper_function(
    planet_hash_str: &str,
    creator_handle: &str,
    config: &Config,
    level_out_dir: &Path,
) -> Result<()> {
    // decode the planet‐hash
    let raw = hex::decode(planet_hash_str)
        .map_err(|e| anyhow!("invalid hex for planet {}: {}", planet_hash_str, e))?;
    if raw.len() != 20 {
        bail!("planet hash must be 20 bytes, got {}", raw.len());
    }
    let mut planet_hash = [0u8; 20];
    planet_hash.copy_from_slice(&raw);

    // 1) download SLTb
    let DownloadResult {
        mut resources,
        success_count,
        error_count,
    } = download_level(
        planet_hash,
        None,
        config.archive_path.to_string_lossy().into_owned(),
        config.max_parallel_downloads,
    )
    .await?;
    println!(
        "Fetched planet {} SLTb: {}/{} blobs",
        planet_hash_str, success_count, error_count
    );

    // 2) parse SLTb for sub‐levels
    let slt_buf = resources
        .get(&planet_hash)
        .ok_or_else(|| anyhow!("planet SLTb missing for {}", planet_hash_str))?;
    let slt_meta = ResrcData::new(slt_buf, false)?;
    let mut deps = Vec::new();
    if let ResrcMethod::Binary { dependencies, .. } = slt_meta.method {
        for d in dependencies {
            if let ResrcDescriptor::Sha1(h) = d.desc {
                deps.push(h);
            }
        }
    }

    // 3) fetch each sub‐level
    for h in deps {
        let DownloadResult {
            resources: lvl_res, ..
        } = download_level(
            h,
            None,
            config.archive_path.to_string_lossy().into_owned(),
            config.max_parallel_downloads,
        )
        .await?;
        for (sha, blob) in lvl_res {
            resources.insert(sha, blob);
        }
        println!("  → added sub‐level {}", hex_encode(h));
    }

    // 4) dump all planet + sub‐level blobs
    for (sha, data) in &resources {
        fs::write(level_out_dir.join(hex_encode(sha)), data)?;
    }

    // 5) write SLTb itself as `<planet_hash>`
    fs::write(
        level_out_dir.join(planet_hash_str),
        resources.get(&planet_hash).unwrap(),
    )?;
    println!("→ wrote planet SLTb blob as {}", planet_hash_str);

    // 6) fetch the creator’s icon BLOB from the user table
    // let (user_icon_blob,): (Vec<u8>,) = Connection::open(&config.database_path)?.query_row(
    //     "SELECT icon FROM \"user\" WHERE npHandle = ?1",
    //     [creator_handle],
    //     |r| Ok((r.get(0)?,)),
    // )?;
    // // name it by its own SHA1 hex
    // let icon_hex = hex_encode(&user_icon_blob);
    // fs::write(level_out_dir.join(&icon_hex), &user_icon_blob)?;
    // println!("→ wrote creator’s icon blob as {}", icon_hex);

    Ok(())
}

async fn fetch_level(level_id: u32, config: &Config) -> Result<()> {
    // 1) Open DB and pull rootLevel, publishedIn, and npHandle
    let conn = Connection::open(&config.database_path)?;
    let (root_blob, published_in, np_handle): (Vec<u8>, Option<String>, String) = conn.query_row(
        r#"
        SELECT rootLevel
             , publishedIn
             , npHandle
          FROM slot
         WHERE id = ?1
        "#,
        [level_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;

    // 2) Sanity-check root_blob
    if root_blob.len() != 20 {
        bail!("slot.rootLevel is {} bytes, expected 20", root_blob.len());
    }
    let mut root_hash = [0u8; 20];
    root_hash.copy_from_slice(&root_blob);

    // 3) Pull slot.icon SHA1
    let icon_sha1_opt: Option<[u8; 20]> =
        conn.query_row("SELECT icon FROM slot WHERE id = ?1", [level_id], |r| {
            let v: Vec<u8> = r.get(0)?;
            if v.len() == 20 {
                let mut a = [0u8; 20];
                a.copy_from_slice(&v);
                Ok(Some(a))
            } else {
                Ok(None)
            }
        })?;

    // 4) Download level blobs (including level-icon)
    let DownloadResult {
        resources,
        success_count,
        error_count,
    } = download_level(
        root_hash,
        icon_sha1_opt,
        config.archive_path.to_string_lossy().into_owned(),
        config.max_parallel_downloads,
    )
    .await?;

    // 5) Dump downloaded blobs
    let out_dir = config.backup_directory.join(format!("level_{}", level_id));
    fs::create_dir_all(&out_dir)?;
    for (sha, data) in &resources {
        fs::write(out_dir.join(hex_encode(sha)), data)?;
    }
    println!(
        "Fetched {} blobs ({}/{}) → {}",
        resources.len(),
        success_count,
        error_count,
        out_dir.display()
    );

    // 6) Recurse parent planet if any
    if let Some(ref parent_hex) = published_in {
        if parent_hex.len() == 40 && parent_hex.chars().all(|c| c.is_ascii_hexdigit()) {
            println!("→ Fetching parent planet {}", parent_hex);
            fetch_planet_resources_helper_function(parent_hex, &np_handle, config, &out_dir)
                .await?;
        }
    }

    // 7) Dump level’s icon (already in `resources`) by SHA1 filename
    if let Some(icon_sha) = icon_sha1_opt {
        if let Some(bytes) = resources.get(&icon_sha) {
            let fname = hex_encode(icon_sha);
            fs::write(out_dir.join(&fname), bytes)?;
            println!("→ wrote level icon blob as {}", fname);
        } else {
            eprintln!(
                "⚠️ icon SHA1 {} not in downloaded resources",
                hex_encode(icon_sha)
            );
        }
    }

    // 8) Pull creator.icon SHA1 + planets list
    let (creator_icon_blob, planets_blob): (Vec<u8>, Vec<u8>) = conn.query_row(
        r#"SELECT icon, planets FROM "user" WHERE npHandle = ?1"#,
        [&np_handle],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    // 9) **Read the creator’s icon directly from your archive**
    // 8) Fetch the creator’s icon via download_level against your local archive
    if creator_icon_blob.len() == 20 {
        let mut creator_hash = [0u8; 20];
        creator_hash.copy_from_slice(&creator_icon_blob);
        // ask download_level to grab exactly that one hash
        let DownloadResult {
            resources: ci_res,
            success_count: _,
            error_count: _,
        } = download_level(
            creator_hash,
            None,
            config.archive_path.to_string_lossy().into_owned(),
            1, // just one
        )
        .await?;
        if let Some(ci_bytes) = ci_res.get(&creator_hash) {
            let fname = hex_encode(creator_hash);
            fs::write(out_dir.join(&fname), ci_bytes)?;
            println!("→ wrote creator icon blob as {}", fname);
        } else {
            eprintln!(
                "⚠️ creator icon SHA1 {} not found in local archive",
                hex_encode(creator_hash)
            );
        }
    } else {
        eprintln!(
            "⚠️ Unexpected creator.icon length: {} bytes (expected 20)",
            creator_icon_blob.len()
        );
    }

    // 10) Recurse creator’s planets
    for chunk in planets_blob.chunks(20) {
        if chunk.len() == 20 {
            let h = hex_encode(chunk);
            println!("→ fetching creator-planet {}", h);
            fetch_planet_resources_helper_function(&h, &np_handle, config, &out_dir).await?;
        }
    }

    // 11) Serialize & RealmImporter
    let users = fetch_all_users(&conn, level_id)?;
    let levels = fetch_all_levels(&conn, level_id)?;
    let relations = fetch_all_relations(&resources);
    let mut assets = fetch_all_assets(&resources);
    let mut dep_map: HashMap<String, Vec<String>> = HashMap::new();
    for r in &relations {
        dep_map
            .entry(r.dependent.clone())
            .or_default()
            .push(r.dependency.clone());
    }
    for a in &mut assets {
        if let Some(d) = dep_map.get(&a.asset_hash) {
            a.dependencies = d.clone();
        }
    }
    let import = ImportData {
        users,
        levels,
        relations,
        assets,
    };
    fs::write("import.json", to_string_pretty(&import)?)?;

    let exe_dir = std::env::current_exe()?.parent().unwrap().to_path_buf();
    Command::new(exe_dir.join("RealmImporter.exe"))
        .arg("template.realm")
        .arg("refreshGameServer.realm")
        .status()?;
    println!("Wrote import.json and produced refreshGameServer.realm");

    Ok(())
}

/// Fetch every level created by `np_handle`
/// by calling `fetch_level` on each slot.id
/// Fetch every level for a creator and dump all blobs into one folder
/// named after their npHandle, skipping duplicate hashes or missing levels.
/// Fetch every level for a creator by calling `fetch_level`, but
/// copy all dumped blobs into one folder named after np_handle.
async fn fetch_entire_planet(np_handle: &str, config: &Config) -> Result<()> {
    // 1) Create the user folder
    let base = config.backup_directory.join(np_handle);
    fs::create_dir_all(&base)?;

    // 2) Query and dedupe level IDs
    let conn = Connection::open(&config.database_path)?;
    let mut stmt = conn.prepare("SELECT id FROM slot WHERE npHandle = ?1")?;
    let mut level_ids: Vec<u32> = stmt
        .query_map([np_handle], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    level_ids.sort_unstable();
    level_ids.dedup();

    if level_ids.is_empty() {
        println!("No levels found for `{}`", np_handle);
        return Ok(());
    }

    // 3) For each level: fetch, then copy its folder contents into `base`
    for lvl in level_ids {
        println!("\n=== Level {} ===", lvl);

        // 3a) run your existing logic (dump + Realm import)
        if let Err(e) = fetch_level(lvl, config).await {
            eprintln!("❌ Skipped level {} due to error: {}", lvl, e);
            continue;
        }

        // 3b) copy files from `level_<id>` into `base`
        let lvl_dir = config.backup_directory.join(format!("level_{}", lvl));
        if !lvl_dir.exists() {
            eprintln!(
                "⚠️  Expected folder {} missing, skipping copy",
                lvl_dir.display()
            );
            continue;
        }
        for entry in fs::read_dir(&lvl_dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let src_path = entry.path();
            let dst_path = base.join(&file_name);

            if dst_path.exists() {
                // skip duplicates
                continue;
            }
            // copy the file
            fs::copy(&src_path, &dst_path).map_err(|e| {
                anyhow!(
                    "failed to copy {} → {}: {}",
                    src_path.display(),
                    dst_path.display(),
                    e
                )
            })?;
        }
    }

    println!(
        "\nAll unique files for `{}` are now in `{}`",
        np_handle,
        base.display()
    );
    Ok(())
}

async fn read_from_file(config: &Config) -> Result<()> {
    // 1) load creators.txt
    let file =
        File::open("creators.txt").map_err(|e| anyhow!("failed to open creators.txt: {}", e))?;
    let creators: Vec<String> = BufReader::new(file)
        .lines()
        .map(|line| {
            let s = line.map_err(|e| anyhow!("read error: {}", e))?;
            let t = s.trim().to_string();
            if t.is_empty() {
                Err(anyhow!("skipping empty line"))
            } else {
                Ok(t)
            }
        })
        .filter_map(Result::ok)
        .collect();

    if creators.is_empty() {
        bail!("creators.txt is empty");
    }

    // 2) find next available fileDumpN
    let mut idx = 0;
    let out_dir: PathBuf = loop {
        let candidate = format!("fileDump{}", idx);
        let path = PathBuf::from(&candidate);
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|e| anyhow!("could not create {}: {}", candidate, e))?;
            break path;
        }
        idx += 1;
    };

    // 3) for each creator: fetch + copy
    for creator in &creators {
        println!("🔄 Fetching entire planet for `{}`…", creator);
        fetch_entire_planet(creator, config).await?;

        let src = config.backup_directory.join(creator);
        if !src.exists() {
            eprintln!("⚠️  no folder for `{}` at {:?}", creator, src);
            continue;
        }
        for entry in fs::read_dir(&src)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let dst = out_dir.join(entry.file_name());
                fs::copy(entry.path(), &dst)
                    .map_err(|e| anyhow!("failed to copy {:?} → {:?}: {}", entry.path(), dst, e))?;
            }
        }
    }

    println!("✅ All files dumped into {:?}", out_dir);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::read()?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Bkp { level_id, lbp3 } => dl_as_backup(level_id, config, lbp3).await?,
        // Commands::Planet { hash } => dl_as_planet(&hash, &config).await?,
        Commands::Planet { hash } => fetch_planet_resources(&hash, &config).await?,
        Commands::FetchLevel { level_id } => match level_id.try_into() {
            Ok(id) => fetch_level(id, &config).await?,
            Err(_) => {
                eprintln!("error: level_id {} is out of range", level_id);
                std::process::exit(1);
            }
        },
        Commands::FetchEntirePlanet { np_handle } => {
            fetch_entire_planet(&np_handle, &config).await?
        }

        Commands::ReadFromFile => read_from_file(&config).await?,
    }

    Ok(())
}
