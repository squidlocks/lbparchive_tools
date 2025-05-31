// Please note : [Backlink] properties and default values are not represented
// in the schema and thus will not be part of the generated models

using System;
using System.Collections.Generic;
using Realms;
using MongoDB.Bson;

namespace MyProject.Models
{
    public class GameUser : RealmObject
    {
        [PrimaryKey]
        public ObjectId UserId { get; set; }

        [Indexed]
        [Required]
        public string Username { get; set; }

        [Indexed]
        public string EmailAddress { get; set; }

        [Indexed]
        public string PasswordBcrypt { get; set; }

        public bool EmailAddressVerified { get; set; }

        public bool ShouldResetPassword { get; set; }

        [Required]
        public string IconHash { get; set; }

        public ObjectId? ForceMatch { get; set; }

        [Required]
        public string PspIconHash { get; set; }

        [Required]
        public string VitaIconHash { get; set; }

        [Required]
        public string BetaIconHash { get; set; }

        public long FilesizeQuotaUsage { get; set; }

        [Required]
        public string Description { get; set; }

        public long LocationX { get; set; }

        public long LocationY { get; set; }

        public DateTimeOffset JoinDate { get; set; }

        public UserPins Pins { get; set; }

        [Required]
        public string BetaPlanetsHash { get; set; }

        [Required]
        public string Lbp2PlanetsHash { get; set; }

        [Required]
        public string Lbp3PlanetsHash { get; set; }

        [Required]
        public string VitaPlanetsHash { get; set; }

        [Required]
        public string YayFaceHash { get; set; }

        [Required]
        public string BooFaceHash { get; set; }

        [Required]
        public string MehFaceHash { get; set; }

        public bool AllowIpAuthentication { get; set; }

        public string BanReason { get; set; }

        public DateTimeOffset? BanExpiryDate { get; set; }

        public DateTimeOffset LastLoginDate { get; set; }

        public bool RpcnAuthenticationAllowed { get; set; }

        public bool PsnAuthenticationAllowed { get; set; }

        public long _ProfileVisibility { get; set; }

        public long _LevelVisibility { get; set; }

        public string PresenceServerAuthToken { get; set; }

        public GamePlaylist RootPlaylist { get; set; }

        public bool UnescapeXmlSequences { get; set; }

        public bool ShowModdedContent { get; set; }

        public long _Role { get; set; }
    }
}
