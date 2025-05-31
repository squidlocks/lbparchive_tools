// Please note : [Backlink] properties and default values are not represented
// in the schema and thus will not be part of the generated models

using System;
using System.Collections.Generic;
using Realms;

namespace MyProject.Models
{
    public class GameLevel : RealmObject
    {
        [PrimaryKey]
        public long LevelId { get; set; }

        public bool IsAdventure { get; set; }

        [Indexed]
        [Required]
        public string Title { get; set; }

        [Required]
        public string IconHash { get; set; }

        [Indexed]
        [Required]
        public string Description { get; set; }

        public long LocationX { get; set; }

        public long LocationY { get; set; }

        [Required]
        public string RootResource { get; set; }

        public DateTimeOffset PublishDate { get; set; }

        public DateTimeOffset UpdateDate { get; set; }

        public long MinPlayers { get; set; }

        public long MaxPlayers { get; set; }

        public bool EnforceMinMaxPlayers { get; set; }

        public bool SameScreenGame { get; set; }

        public DateTimeOffset? DateTeamPicked { get; set; }

        public bool IsModded { get; set; }

        public string BackgroundGuid { get; set; }

        public long _GameVersion { get; set; }

        public long _LevelType { get; set; }

        [Indexed]
        public long StoryId { get; set; }

        public bool IsLocked { get; set; }

        public bool IsSubLevel { get; set; }

        public bool IsCopyable { get; set; }

        public float Score { get; set; }

        public IList<GameSkillReward> _SkillRewards { get; }

        public IList<GameReview> Reviews { get; }

        public GameUser Publisher { get; set; }

        public string OriginalPublisher { get; set; }

        public bool IsReUpload { get; set; }
    }
}
