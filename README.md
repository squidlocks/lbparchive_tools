# LBP Archive Tools

A command-line toolkit for downloading, processing, and backing up LittleBigPlanet levels and planets. This tool interacts with a local resource archive to fetch LBP resources (levels, icons, planets), assemble save archives, and prepare import data.

---

## Features

- **Level Backup (`bkp`)**  
  Download all resources for a given level ID and create a complete LBP save-archive backup (including ICON0.PNG, save archive chunks, PARAM.SFO, PARAM.PFD).

- **Planet Download (`planet`)**  
  Download every resource blob for a given 40-hex SHA1 rootLevel of a planet. Produces an LBP-compatible filesystem structure for that planet.

- **Fetch Single Level (`fetch-level`)**  
  Given a numeric level ID, download its rootLevel and icon, dump all blobs to `backup_directory/level_<id>/`, and write out related metadata (import.json, RealmImporter output).

- **Fetch Entire Planet (`fetch-entire-planet`)**  
  Given an LBP creator’s NP handle, download every level they created, then copy all unique blobs into a single folder named after that NP handle. Useful for bulk extraction of a creator’s entire “planet.”

- **Read from File (`read-from-file`)**  
  Given a `creators.txt` (one NP handle per line), automatically run `fetch-entire-planet` for each and consolidate all resulting blobs into a `fileDump<index>/` folder.

---

## Prerequisites

LittleBigRefresh server version (v2.18.0) is only supported at this time. 

1. **Rust & Cargo**  
   - Install from [rustup.rs](https://rustup.rs/) or your OS package manager.  
   - Minimum Rust version: 1.60 (latest stable recommended).

2. **.NET SDK (8.0 or later)**  
   - Required to build `RealmImporter` (written in .NET).  
   - Download from [.NET official site](https://dotnet.microsoft.com/download).  

3. **SQLite & Development Headers** (if on Linux/macOS)  
   - The tool uses `rusqlite` to query the local LBP database. Ensure you have `sqlite3` and its development headers installed.  
     - **Ubuntu/Debian**:  
       ```bash
       sudo apt-get install sqlite3 libsqlite3-dev
       ```
     - **macOS** (with Homebrew):  
       ```bash
       brew install sqlite3
       ```

4. **LBP Local Archive**  
   - A directory containing LBP resource ZIPs or a remote server (e.g. archive.org). See [Configuration](#configuration) for details.

5. **A Config File (`config.yml`)**  
   - Describes database path, backup directory, download server or local archive path, and other settings.

6. **Realm File From Server** 
   - copy and paste your .realm file into where the exe's are located from your LBP Server and rename it as template.realm
---

## Configuration

Create a `config.yml` in the project root. Example:

```yaml
# Path to the SQLite database file (download dry.db from archive, if you need it)
database_path: "dry.db"

# Where backups and level folders will be written
backup_directory: "backups"

# Choose "refresh" (HTTP) or "archive" (online ZIPs) for remote downloads:
#   refresh → https://lbp.littlebigrefresh.com/
#   archive → https://archive.org/details/@tamiya99
# Only used if `online: 1` (see below).
download_server: "refresh"

# Base path for a local ZIP archive (used when online: 0)
archive_path: "D:\\LBP Archive"

# Toggle between ZIP-based (local) or HTTP-based (online) fetching:
#   0 → use local archive (via archive_path)
#   1 → use HTTP (download_server)
online: 0

# Maximum parallel downloads (1–10 recommended)
max_parallel_downloads: 10

# If true, levels in LBP1/2 format will still be backed up as LBP3.
fix_backup_version: true

# If true, *all* LBP1/LBP2 levels are forced to LBP3 backups (overrides fix_backup_version)
force_lbp3_backups: false


### Usage

Once you have built both the Rust CLI and `RealmImporter.exe`, examples below assume:

- You’re in the root of the Rust project.
- `archive_dl` (the Rust binary) and `RealmImporter.exe` are both in your `PATH` or in the current directory.

```bash
# On Windows:
archive_dl.exe <command> [arguments]

# On Linux/macOS:
archive_dl <command> [arguments]
```

Run `--help` for an overview:

```bash
archive_dl --help
```

You’ll see:

```
archive_dl 1.0
Command-line tools for LittleBigPlanet resource backup and retrieval

USAGE:
    archive_dl <COMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

COMMANDS:
    bkp                 Download level and save as level backup
    planet              Download resources for a planet (40-hex SHA1)
    fetch-level         Download a single level by numeric ID
    fetch-entire-planet Fetch all levels for a creator (by NP handle)
    read-from-file      Read NP handles from creators.txt and fetch all planets
    help                Prints this message or the help of the given subcommand
```

#### `bkp` (Backup a single level)

```bash
archive_dl bkp <level_id> [--lbp3]
```

- `<level_id>`: Numeric ID from your SQLite `slot` table.
- `--lbp3`: Force backup format to LBP3 even if the level is older.

Example:

```bash
# Backup level ID 1234 in its native format:
archive_dl bkp 1234

# Force LBP3 backup for level ID 1234:
archive_dl bkp 1234 --lbp3
```

---

#### `planet` (Download a planet’s SLTb and all level blobs)

```bash
archive_dl planet <planet_sha1>
```

- `<planet_sha1>`: A 40-hex SHA1 string (e.g. `3622E8A1234567890ABCDEF1234567890ABCDEF`) for the planet’s rootLevel.

Example:

```bash
archive_dl planet 3622E8A1234567890ABCDEF1234567890ABCDEF
```

---

#### `fetch-level` (Fetch & dump a single level by ID)

```bash
archive_dl fetch-level <level_id>
```

- `<level_id>`: Numeric ID from the SQLite `slot` table.

Example:

```bash
archive_dl fetch-level 1234
```

---

#### `fetch-entire-planet` (Fetch all levels for a creator)

```bash
archive_dl fetch-entire-planet <np_handle>
```

- `<np_handle>`: The LBP creator’s PlayStation Network handle.

Example:

```bash
archive_dl fetch-entire-planet CyriusTheVirus
```

---

#### `read-from-file` (Batch fetch from `creators.txt`)

```bash
archive_dl read-from-file
```

- No arguments.
- Reads `creators.txt` (one NP handle per line).
- Creates `fileDump0/`, `fileDump1/`, etc., for each time you run it.

Example:

```bash
# Given creators.txt contains:
# CyriusTheVirus
# another_user

archive_dl read-from-file
```

CREDITS TO [lbp_archive_dl by uhwot](https://github.com/uhwot/lbp_archive_dl) for allowing me to fork his code!
