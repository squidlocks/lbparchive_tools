// src/resource_dl.rs

use anyhow::{Result, anyhow};
use dashmap::DashMap;
use sha1::{Digest, Sha1};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    path::{Path, PathBuf},
    sync::{Arc, Mutex as StdMutex},
    time::Instant,
};
use tokio::{
    sync::{Mutex as AsyncMutex, Semaphore},
    task::{JoinSet, spawn_blocking},
};
use zip::ZipArchive;
use crate::resource_parse::{ResrcData, ResrcDescriptor, ResrcMethod};

pub struct DownloadResult {
    pub resources: BTreeMap<[u8; 20], Vec<u8>>,
    pub success_count: usize,
    pub error_count: usize,
}

#[derive(Clone)]
struct Downloader {
    seen: Arc<AsyncMutex<BTreeSet<[u8; 20]>>>,
    cache: Arc<AsyncMutex<BTreeMap<[u8; 20], Vec<u8>>>>,
    sem: Arc<Semaphore>,
    zip_pool: Arc<DashMap<PathBuf, StdMutex<ZipArchive<File>>>>,
    cache_dir: PathBuf,
}

impl Downloader {
    /// Build a new Downloader.
    pub fn new(max_parallel: usize, cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cache_dir)
            .map_err(|e| anyhow!("couldn't create cache dir `{}`: {}", cache_dir.display(), e))?;
        Ok(Self {
            seen: Arc::new(AsyncMutex::new(BTreeSet::new())),
            cache: Arc::new(AsyncMutex::new(BTreeMap::new())),
            sem: Arc::new(Semaphore::new(max_parallel)),
            zip_pool: Arc::new(DashMap::new()),
            cache_dir,
        })
    }

    /// Fetch one SHA1, using on‐disk cache, in‐memory cache, or opening the right ZIP.
    pub async fn fetch_one_cached(
        self: Arc<Self>,
        sha1: [u8; 20],
        archive_root: PathBuf,
    ) -> Result<Vec<[u8; 20]>> {
        // hex string for logging & cache filename
        let hex = hex::encode(sha1);
        let cache_file = self.cache_dir.join(&hex);

        // 1) on‐disk cache hit?
        if cache_file.exists() {
            eprintln!("▶ [cache hit] {}", hex);
            let buf = fs::read(&cache_file)?;
            let mut hasher = Sha1::new(); hasher.update(&buf);
            if hasher.finalize().as_slice() != sha1 {
                return Err(anyhow!("SHA1 mismatch on cache {}", hex));
            }
            {
                let mut seen = self.seen.lock().await;
                if !seen.insert(sha1) {
                    return Ok(vec![]);
                }
            }
            {
                let mut mem = self.cache.lock().await;
                mem.insert(sha1, buf.clone());
            }
            let meta = ResrcData::new(&buf, false)?;
            if let ResrcMethod::Binary { dependencies, .. } = meta.method {
                return Ok(dependencies.into_iter()
                    .filter_map(|d| if let ResrcDescriptor::Sha1(s) = d.desc { Some(s) } else { None })
                    .collect());
            } else {
                return Ok(vec![]);
            }
        }

        // 2) otherwise: derive the ZIP path & entry
        let first       = u8::from_str_radix(&hex[0..2], 16).unwrap();
        let range_start = first & 0xF0;
        let range_end   = range_start | 0x0F;
        let res_folder  = format!("LBP online levels 2023 (res {:02x}-{:02x})", range_start, range_end);
        let subfolder   = format!("dry23r{}", &hex[0..1]);
        let zipname     = format!("dry{}.zip", &hex[0..2]);
        let zip_path    = archive_root.join(&res_folder).join(&subfolder).join(&zipname);
        let entry_name  = format!("{}/{}/{}", &hex[0..2], &hex[2..4], hex);

        eprintln!("▶ Fetching resources from {}", zipname);
        let _permit = self.sem.acquire().await?;

        // clone hex so we don't move the original
        let hex_for_spawn = hex.clone();
        let (buf, deps) = spawn_blocking({
            let pool = self.zip_pool.clone();
            move || -> Result<(Vec<u8>, Vec<[u8; 20]>)> {
                // open or reuse the zip
                if pool.get(&zip_path).is_none() {
                    let f = File::open(&zip_path)
                        .map_err(|e| anyhow!("couldn't open {}: {}", zip_path.display(), e))?;
                    let arch = ZipArchive::new(f)
                        .map_err(|e| anyhow!("{} not a zip: {}", zip_path.display(), e))?;
                    pool.insert(zip_path.clone(), StdMutex::new(arch));
                }
                let mutex = pool.get(&zip_path).unwrap();
                let mut archive = mutex.lock()
                    .map_err(|e| anyhow!("mutex poisoned for {}: {}", zip_path.display(), e))?;

                // extract entry
                let mut zf = archive
                    .by_name(&entry_name)
                    .map_err(|e| anyhow!("{} missing {}: {}", zip_path.display(), entry_name, e))?;
                let mut buf = Vec::with_capacity(zf.size() as usize);
                std::io::copy(&mut zf, &mut buf)?;

                // verify & parse deps
                let mut hasher = Sha1::new(); hasher.update(&buf);
                if hasher.finalize().as_slice() != sha1 {
                    return Err(anyhow!("SHA1 mismatch for {}", hex_for_spawn));
                }
                let meta = ResrcData::new(&buf, false)?;
                let deps = if let ResrcMethod::Binary { dependencies, .. } = meta.method {
                    dependencies.into_iter()
                        .filter_map(|d| if let ResrcDescriptor::Sha1(s) = d.desc { Some(s) } else { None })
                        .collect()
                } else {
                    Vec::new()
                };
                Ok((buf, deps))
            }
        })
        .await??;

        // 3) cache to disk
        fs::write(&cache_file, &buf)?;

        // 4) in‐memory record & return deps
        {
            let mut seen = self.seen.lock().await;
            if !seen.insert(sha1) {
                return Ok(vec![]);
            }
        }
        {
            let mut mem = self.cache.lock().await;
            mem.insert(sha1, buf.clone());
        }
        eprintln!("\tgot file: {}", hex);

        Ok(deps)
    }
}

/// Public entrypoint
pub async fn download_level(
    root: [u8; 20],
    icon_sha1: Option<[u8; 20]>,
    archive_root: String,
    max_parallel: usize,
) -> Result<DownloadResult> {
    let start = Instant::now();
    let root_dir = PathBuf::from(&archive_root);

    // cache next to exe
    let exe_path = std::env::current_exe()
        .map_err(|e| anyhow!("couldn't find exe path: {}", e))?;
    let exe_dir = exe_path
        .parent()
        .ok_or_else(|| anyhow!("exe has no parent directory"))?;
    let cache_dir = exe_dir.join("resource_cache");

    let dl = Arc::new(Downloader::new(max_parallel, cache_dir)?);
    let mut js = JoinSet::new();

    // enqueue root
    {
        let dl0 = dl.clone();
        let rd0 = root_dir.clone();
        js.spawn(async move { dl0.fetch_one_cached(root, rd0).await });
    }
    // optionally icon
    if let Some(ic) = icon_sha1 {
        let dl1 = dl.clone();
        let rd1 = root_dir.clone();
        js.spawn(async move { dl1.fetch_one_cached(ic, rd1).await });
    }

    // process deps
    let mut pending = BTreeSet::new();
    pending.insert(root);
    if let Some(ic) = icon_sha1 {
        pending.insert(ic);
    }
    while let Some(res) = js.join_next().await {
        let deps = res??;
        for child in deps {
            if pending.insert(child) {
                let dlc = dl.clone();
                let rdc = root_dir.clone();
                js.spawn(async move { dlc.fetch_one_cached(child, rdc).await });
            }
        }
    }

    // collect
    let mut guard = dl.cache.lock().await;
    let resources = std::mem::take(&mut *guard);

    eprintln!("▶ All resources fetched in {:.2?}", start.elapsed());
    Ok(DownloadResult {
        success_count: resources.len(),
        error_count: 0,
        resources,
    })
}
