// Please note : [Backlink] properties and default values are not represented
// in the schema and thus will not be part of the generated models

using System;
using System.Collections.Generic;
using Realms;

namespace MyProject.Models
{
    public class GameContest : RealmObject
    {
        [PrimaryKey]
        public string ContestId { get; set; }

        public GameUser Organizer { get; set; }

        public DateTimeOffset CreationDate { get; set; }

        public DateTimeOffset StartDate { get; set; }

        public DateTimeOffset EndDate { get; set; }

        public string ContestTag { get; set; }

        public string BannerUrl { get; set; }

        public string ContestTitle { get; set; }

        public string ContestSummary { get; set; }

        public string ContestDetails { get; set; }

        public string ContestTheme { get; set; }

        public IList<long> _AllowedGames { get; }

        public GameLevel TemplateLevel { get; set; }
    }
}
