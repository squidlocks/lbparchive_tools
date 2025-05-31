// src/models.rs

use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Top‐level wrapper for your import.json
#[derive(Serialize)]
pub struct ImportData {
    pub users: Vec<GameUser>,
    pub levels: Vec<GameLevel>,
    pub relations: Vec<AssetDependencyRelation>,
    pub assets: Vec<GameAsset>,
}

/// Mirrors your C# GameUser
#[derive(Serialize)]
pub struct GameUser {
    #[serde(rename = "UserId")]
    pub user_id: ObjectId,

    #[serde(rename = "Username")]
    pub username: String,

    #[serde(rename = "EmailAddress")]
    pub email_address: Option<String>,

    #[serde(rename = "PasswordBcrypt")]
    pub password_bcrypt: Option<String>,

    #[serde(rename = "EmailAddressVerified")]
    pub email_address_verified: bool,

    #[serde(rename = "ShouldResetPassword")]
    pub should_reset_password: bool,

    #[serde(rename = "IconHash")]
    pub icon_hash: String,

    #[serde(rename = "ForceMatch")]
    pub force_match: Option<ObjectId>,

    #[serde(rename = "PspIconHash")]
    pub psp_icon_hash: String,

    #[serde(rename = "VitaIconHash")]
    pub vita_icon_hash: String,

    #[serde(rename = "BetaIconHash")]
    pub beta_icon_hash: String,

    #[serde(rename = "FilesizeQuotaUsage")]
    pub filesize_quota_usage: i64,

    #[serde(rename = "Description")]
    pub description: String,

    #[serde(rename = "LocationX")]
    pub location_x: i64,

    #[serde(rename = "LocationY")]
    pub location_y: i64,

    #[serde(rename = "JoinDate")]
    pub join_date: DateTime<Utc>,

    #[serde(rename = "Pins")]
    pub pins: Value, // placeholder for UserPins

    #[serde(rename = "BetaPlanetsHash")]
    pub beta_planets_hash: String,

    #[serde(rename = "Lbp2PlanetsHash")]
    pub lbp2_planets_hash: String,

    #[serde(rename = "Lbp3PlanetsHash")]
    pub lbp3_planets_hash: String,

    #[serde(rename = "VitaPlanetsHash")]
    pub vita_planets_hash: String,

    #[serde(rename = "YayFaceHash")]
    pub yay_face_hash: String,

    #[serde(rename = "BooFaceHash")]
    pub boo_face_hash: String,

    #[serde(rename = "MehFaceHash")]
    pub meh_face_hash: String,

    #[serde(rename = "AllowIpAuthentication")]
    pub allow_ip_authentication: bool,

    #[serde(rename = "BanReason")]
    pub ban_reason: Option<String>,

    #[serde(rename = "BanExpiryDate")]
    pub ban_expiry_date: Option<DateTime<Utc>>,

    #[serde(rename = "LastLoginDate")]
    pub last_login_date: DateTime<Utc>,

    #[serde(rename = "RpcnAuthenticationAllowed")]
    pub rpcn_authentication_allowed: bool,

    #[serde(rename = "PsnAuthenticationAllowed")]
    pub psn_authentication_allowed: bool,

    #[serde(rename = "_ProfileVisibility")]
    pub _profile_visibility: i64,

    #[serde(rename = "_LevelVisibility")]
    pub _level_visibility: i64,

    #[serde(rename = "PresenceServerAuthToken")]
    pub presence_server_auth_token: Option<String>,

    #[serde(rename = "RootPlaylist")]
    pub root_playlist: Value, // placeholder for GamePlaylist

    #[serde(rename = "UnescapeXmlSequences")]
    pub unescape_xml_sequences: bool,

    #[serde(rename = "ShowModdedContent")]
    pub show_modded_content: bool,

    #[serde(rename = "_Role")]
    pub _role: i64,
}

/// Mirrors your C# GameLevel
#[derive(Serialize)]
pub struct GameLevel {
    #[serde(rename = "LevelId")]
    pub level_id: i64,

    #[serde(rename = "IsAdventure")]
    pub is_adventure: bool,

    #[serde(rename = "Title")]
    pub title: String,

    #[serde(rename = "IconHash")]
    pub icon_hash: String,

    #[serde(rename = "Description")]
    pub description: String,

    #[serde(rename = "LocationX")]
    pub location_x: i64,

    #[serde(rename = "LocationY")]
    pub location_y: i64,

    #[serde(rename = "RootResource")]
    pub root_resource: String,

    #[serde(rename = "PublishDate")]
    pub publish_date: DateTime<Utc>,

    #[serde(rename = "UpdateDate")]
    pub update_date: DateTime<Utc>,

    #[serde(rename = "MinPlayers")]
    pub min_players: i64,

    #[serde(rename = "MaxPlayers")]
    pub max_players: i64,

    #[serde(rename = "EnforceMinMaxPlayers")]
    pub enforce_min_max_players: bool,

    #[serde(rename = "SameScreenGame")]
    pub same_screen_game: bool,

    #[serde(rename = "DateTeamPicked")]
    pub date_team_picked: Option<DateTime<Utc>>,

    #[serde(rename = "IsModded")]
    pub is_modded: bool,

    #[serde(rename = "BackgroundGuid")]
    pub background_guid: Option<String>,

    #[serde(rename = "_GameVersion")]
    pub _game_version: i64,

    #[serde(rename = "_LevelType")]
    pub _level_type: i64,

    #[serde(rename = "StoryId")]
    pub story_id: i64,

    #[serde(rename = "IsLocked")]
    pub is_locked: bool,

    #[serde(rename = "IsSubLevel")]
    pub is_sub_level: bool,

    #[serde(rename = "IsCopyable")]
    pub is_copyable: bool,

    #[serde(rename = "Score")]
    pub score: f32,

    #[serde(rename = "_SkillRewards")]
    pub skill_rewards: Vec<Value>, // placeholder for GameSkillReward

    #[serde(rename = "Reviews")]
    pub reviews: Vec<Value>, // placeholder for GameReview

    #[serde(rename = "Publisher")]
    pub publisher_id: ObjectId,

    #[serde(rename = "OriginalPublisher")]
    pub original_publisher: Option<String>,

    #[serde(rename = "IsReUpload")]
    pub is_re_upload: bool,
    


    
}

/// Mirrors your C# AssetDependencyRelation
#[derive(Serialize)]
pub struct AssetDependencyRelation {
    #[serde(rename = "Dependent")]
    pub dependent: String,

    #[serde(rename = "Dependency")]
    pub dependency: String,
}

/// Mirrors your C# GameAsset
#[derive(Serialize)]
pub struct GameAsset {
    #[serde(rename = "AssetHash")]
    pub asset_hash: String,

    #[serde(rename = "OriginalUploader")]
    pub original_uploader_id: ObjectId,

    #[serde(rename = "UploadDate")]
    pub upload_date: DateTime<Utc>,

    #[serde(rename = "IsPSP")]
    pub is_psp: bool,

    #[serde(rename = "SizeInBytes")]
    pub size_in_bytes: i64,

    #[serde(rename = "_AssetType")]
    pub _asset_type: i64,

    #[serde(rename = "_AssetSerializationMethod")]
    pub _asset_serialization_method: i64,

    #[serde(rename = "Dependencies")]
    pub dependencies: Vec<String>,

    #[serde(rename = "AsMainlineIconHash")]
    pub as_mainline_icon_hash: Option<String>,

    #[serde(rename = "AsMipIconHash")]
    pub as_mip_icon_hash: Option<String>,

    #[serde(rename = "AsMainlinePhotoHash")]
    pub as_mainline_photo_hash: Option<String>,
}
