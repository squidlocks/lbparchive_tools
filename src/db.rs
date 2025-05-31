// src/db.rs

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::Path;

use anyhow::{Result, anyhow};
use bitvec::{order::Lsb0, view::BitView};
use rusqlite::{Connection, params};

use crate::resource_parse::ResrcData;
use crate::resource_parse::ResrcMethod;
use crate::{ResrcDescriptor, labels::LABEL_LAMS_KEY_IDS, resource_parse::ResrcRevision};

use crate::models::{AssetDependencyRelation, GameAsset, GameLevel, GameUser};
use bson::oid::ObjectId;
use chrono::{DateTime, TimeZone, Utc};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum GameVersion {
    Lbp1,
    Lbp2,
    Lbp3,
}

impl GameVersion {
    pub fn get_title(&self) -> &'static str {
        match self {
            Self::Lbp1 => "LittleBigPlanet™",
            Self::Lbp2 => "LittleBigPlanet™2",
            Self::Lbp3 => "LittleBigPlanet™3",
        }
    }
    pub fn get_short_title(&self) -> &'static str {
        match self {
            Self::Lbp1 => "LBP1",
            Self::Lbp2 => "LBP2",
            Self::Lbp3 => "LBP3",
        }
    }
    pub fn get_titleid(&self) -> &'static str {
        match self {
            Self::Lbp1 => "BCES00141",
            Self::Lbp2 => "BCES00850",
            Self::Lbp3 => "BCES01663",
        }
    }
    pub fn get_latest_revision(&self) -> ResrcRevision {
        match self {
            Self::Lbp1 => ResrcRevision {
                head: 0x272,
                branch_id: 0x4c44,
                branch_revision: 0x17,
            },
            Self::Lbp2 => ResrcRevision {
                head: 0x3f8,
                branch_id: 0x0,
                branch_revision: 0x0,
            },
            Self::Lbp3 => ResrcRevision {
                head: 0x21803f9,
                branch_id: 0x0,
                branch_revision: 0x0,
            },
        }
    }
}

#[derive(Debug)]
pub enum LevelType {
    Cooperative,
    Versus,
    Cutscene,
}

#[derive(Debug)]
pub struct SlotInfo {
    pub name: String,
    pub description: String,
    pub np_handle: String,
    pub root_level: [u8; 20],
    pub icon: ResrcDescriptor,
    pub game: GameVersion,
    pub initially_locked: bool,
    pub is_sub_level: bool,
    pub background_guid: Option<u32>,
    pub shareable: bool,
    pub author_labels: Vec<u32>,
    pub leveltype: LevelType,
    pub min_players: Option<u8>,
    pub max_players: Option<u8>,
    pub is_adventure_planet: bool,
}

pub fn get_slot_info(id: i64, db_path: &Path) -> Result<SlotInfo> {
    // 1) make sure file exists
    if !db_path.exists() {
        return Err(anyhow!(
            "Database file is missing, download it or check if the path in config.yml is correct"
        ));
    }

    // 2) open with rusqlite
    let conn = Connection::open(db_path)
        .map_err(|e| anyhow!("Failed to open DB {}: {}", db_path.display(), e))?;

    // 3) prepare & execute exactly one row
    let mut stmt = conn.prepare(
        "SELECT
            name,
            description,
            npHandle,
            rootLevel,
            icon,
            game,
            initiallyLocked,
            isSubLevel,
            background,
            shareable,
            authorLabels,
            leveltype,
            minPlayers,
            maxPlayers,
            isAdventurePlanet
         FROM slot WHERE id = ?1",
    )?;

    let mut rows = stmt.query(params![id])?;
    let row = rows.next()?.ok_or_else(|| anyhow!("Level not found"))?;

    // 4) pull out every column just like before
    let name: String = row.get::<_, Option<String>>(0)?.unwrap_or_default();
    let description: String = row.get::<_, Option<String>>(1)?.unwrap_or_default();
    let np_handle: String = row.get(2)?;

    // rootLevel blob → [u8;20]
    let raw_root: Vec<u8> = row.get(3)?;
    let root_level: [u8; 20] = raw_root
        .try_into()
        .map_err(|_| anyhow!("invalid rootLevel in db"))?;

    // icon blob → Sha1 or Guid
    let raw_icon: Vec<u8> = row.get(4)?;
    let icon = match raw_icon.len() {
        20 => {
            let mut arr = [0u8; 20];
            arr.copy_from_slice(&raw_icon);
            ResrcDescriptor::Sha1(arr)
        }
        4 => {
            let mut arr = [0u8; 4];
            arr.copy_from_slice(&raw_icon);
            ResrcDescriptor::Guid(u32::from_be_bytes(arr))
        }
        _ => return Err(anyhow!("invalid icon in db")),
    };

    // game version
    let game_int: i64 = row.get(5)?;
    let game = match game_int {
        0 => GameVersion::Lbp1,
        1 => GameVersion::Lbp2,
        2 => GameVersion::Lbp3,
        other => return Err(anyhow!("invalid game version `{}` in db", other)),
    };

    // bool flags
    let initially_locked: bool = row.get::<_, i64>(6)? != 0;
    let is_sub_level: bool = row.get::<_, i64>(7)? != 0;

    // optional background
    let background_guid: Option<u32> = row.get::<_, Option<i64>>(8)?.map(|i| i as u32);

    let shareable: bool = row.get::<_, i64>(9)? != 0;

    // bitfield blob → Vec<u32>
    let mut author_labels = Vec::with_capacity(5);
    if let Some(bits_blob) = row.get::<_, Option<Vec<u8>>>(10)? {
        let bits = bits_blob.view_bits::<Lsb0>();
        for (i, key_id) in LABEL_LAMS_KEY_IDS.iter().enumerate() {
            // dereference the BitRef to a bool
            if bits.get(i).map(|b| *b).unwrap_or(false) {
                author_labels.push(*key_id);
            }
        }
    }

    // leveltype
    let lt: Option<String> = row.get(11)?;
    let leveltype = match lt.as_deref() {
        None => LevelType::Cooperative,
        Some("versus") => LevelType::Versus,
        Some("cutscene") => LevelType::Cutscene,
        Some(other) => return Err(anyhow!("invalid leveltype `{}` in db", other)),
    };

    // min/max players
    let min_players: Option<u8> = row.get::<_, Option<i64>>(12)?.map(|i| i as u8);
    let max_players: Option<u8> = row.get::<_, Option<i64>>(13)?.map(|i| i as u8);

    let is_adventure_planet: bool = row.get::<_, i64>(14)? != 0;

    Ok(SlotInfo {
        name,
        description,
        np_handle,
        root_level,
        icon,
        game,
        initially_locked,
        is_sub_level,
        background_guid,
        shareable,
        author_labels,
        leveltype,
        min_players,
        max_players,
        is_adventure_planet,
    })
}

pub fn fetch_all_users(conn: &Connection, level_id: u32) -> Result<Vec<GameUser>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
          u.npHandle,
          u.icon,
          u.locationX,
          u.locationY,
          u.commentsEnabled,
          u.planets
        FROM "user" AS u
        INNER JOIN slot AS s
          ON s.npHandle = u.npHandle
        WHERE s.id = ?1
        "#,
    )?;

    let users = stmt
        .query_map(params![level_id], |row| {
            let user_id = ObjectId::new();

            // fetch npHandle → Username
            let username = row.get::<_, String>(0)?;

            // icon blob
            let icon_blob: Vec<u8> = row.get(1)?;
            let icon_hash = hex::encode(&icon_blob);

            // location
            let location_x = row.get::<_, u16>(2)? as i64;
            let location_y = row.get::<_, u16>(3)? as i64;

            // commentsEnabled
            let allow_ip_auth = row.get::<_, i64>(4)? != 0;

            // ** NEW ** planets blob → lbp2_planets_hash
            let planets_blob: Vec<u8> = row.get(5)?;
            let lbp2_planets_hash = hex::encode(&planets_blob);

            Ok(GameUser {
                user_id,
                username,
                icon_hash,
                // leave EmailAddress etc empty for now
                email_address: Some(String::new()),
                password_bcrypt: Some(String::new()),
                email_address_verified: false,
                should_reset_password: false,
                force_match: None,
                psp_icon_hash: String::new(),
                vita_icon_hash: String::new(),
                beta_icon_hash: String::new(),
                filesize_quota_usage: 0,
                description: String::new(),
                location_x,
                location_y,
                join_date: Utc.timestamp_opt(0, 0).unwrap(),
                pins: Default::default(),
                beta_planets_hash: String::new(),
                lbp2_planets_hash, // ← filled now!
                lbp3_planets_hash: String::new(),
                vita_planets_hash: String::new(),
                yay_face_hash: String::new(),
                boo_face_hash: String::new(),
                meh_face_hash: String::new(),
                allow_ip_authentication: allow_ip_auth,
                ban_reason: None,
                ban_expiry_date: None,
                last_login_date: Utc.timestamp_opt(0, 0).unwrap(),
                rpcn_authentication_allowed: false,
                psn_authentication_allowed: false,
                _profile_visibility: 0,
                _level_visibility: 0,
                presence_server_auth_token: None,
                root_playlist: Default::default(),
                unescape_xml_sequences: false,
                show_modded_content: false,
                _role: 0,
            })
        })?
        .collect::<Result<_, _>>()?;

    Ok(users)
}

/// Fetch exactly this one GameLevel

pub fn fetch_all_levels(conn: &Connection, level_id: u32) -> Result<Vec<GameLevel>> {
    // 1) pull exactly this slot row
    let mut stmt = conn.prepare(
        r#"
        SELECT 
            id, 
            isAdventurePlanet,      -- we’ll treat this as IsAdventure
            name, 
            icon, 
            description,
            locationX, 
            locationY, 
            rootLevel,
            firstPublished,
            lastUpdated,
            minPlayers,
            maxPlayers
        FROM slot
        WHERE id = ?1
    "#,
    )?;

    let level = stmt.query_row(params![level_id], |row| {
        // helper to turn UNIX‐ms → chrono DateTime<Utc>
        fn ms_to_dt(ms: Option<u64>) -> DateTime<Utc> {
            let ms = ms.unwrap_or(0);
            let secs = (ms / 1_000) as i64;
            let nsec = ((ms % 1_000) * 1_000_000) as u32;
            Utc.timestamp_opt(secs, nsec).unwrap()
        }

        // pull out the bits
        let id: u32 = row.get(0)?;
        let is_adv: bool = row.get::<_, i64>(1)? != 0;
        let name: Option<String> = row.get(2)?;
        let icon_blob: Vec<u8> = row.get(3)?;
        let desc: Option<String> = row.get(4)?;
        let lx: u16 = row.get(5)?;
        let ly: u16 = row.get(6)?;
        let root_blob: Vec<u8> = row.get(7)?;
        let first_pub: Option<u64> = row.get(8)?;
        let last_upd: Option<u64> = row.get(9)?;
        let min_p: Option<u8> = row.get::<_, Option<i64>>(10)?.map(|i| i as u8);
        let max_p: Option<u8> = row.get::<_, Option<i64>>(11)?.map(|i| i as u8);

        // map into your RealmObject struct
        Ok(GameLevel {
            level_id: id as i64,
            is_adventure: is_adv,
            title: name.unwrap_or_default(),
            icon_hash: hex::encode(icon_blob),
            description: desc.unwrap_or_default(),
            location_x: lx as i64,
            location_y: ly as i64,
            root_resource: hex::encode(root_blob),
            publish_date: ms_to_dt(first_pub),
            update_date: ms_to_dt(last_upd),
            min_players: min_p.unwrap_or(0) as i64,
            max_players: max_p.unwrap_or(0) as i64,
            enforce_min_max_players: false, // placeholder
            same_screen_game: false,
            date_team_picked: None,
            is_modded: false,
            background_guid: None,
            _game_version: 0,
            _level_type: 0,
            story_id: 0,
            is_locked: false,
            is_sub_level: false,
            is_copyable: false,
            score: 0.0,
            skill_rewards: Vec::new(),
            reviews: Vec::new(),
            publisher_id: ObjectId::new(), // we’ll wire this up from fetch_all_users
            original_publisher: Some(String::new()),
            is_re_upload: false,
        })
    })?;

    Ok(vec![level])
}

pub fn fetch_all_relations(
    resources: &BTreeMap<[u8; 20], Vec<u8>>,
) -> Vec<AssetDependencyRelation> {
    let mut rels = Vec::new();

    for (parent_sha, blob) in resources {
        // try to parse it as a ResrcData
        if let Ok(resrc) = ResrcData::new(blob, /* do_decompress */ false) {
            if let ResrcMethod::Binary { dependencies, .. } = resrc.method {
                for dep in dependencies {
                    // only Sha1‐desc dependencies are real blobs
                    if let ResrcDescriptor::Sha1(child_sha) = dep.desc {
                        rels.push(AssetDependencyRelation {
                            dependent: hex::encode(parent_sha),
                            dependency: hex::encode(&child_sha),
                        });
                    }
                }
            }
        }
    }

    rels
}

/// Fetch all GameAsset rows *for* this level
pub fn fetch_all_assets(resources: &BTreeMap<[u8; 20], Vec<u8>>) -> Vec<GameAsset> {
    resources
        .iter()
        .map(|(sha, _blob)| {
            GameAsset {
                asset_hash: hex::encode(sha),
                // dry.db doesn’t have uploader ObjectIds, so just make a new one:
                original_uploader_id: ObjectId::new(),
                // we don’t know the real upload date yet:
                upload_date: Utc.timestamp_opt(0, 0).unwrap(),
                is_psp: false,
                size_in_bytes: 0,
                _asset_type: 0,
                _asset_serialization_method: 0,
                // will fill from your relations map:
                dependencies: Vec::new(),
                as_mainline_icon_hash: Some(String::new()),
                as_mip_icon_hash: Some(String::new()),
                as_mainline_photo_hash: Some(String::new()),
            }
        })
        .collect()
}
