// Please note : [Backlink] properties and default values are not represented
// in the schema and thus will not be part of the generated models

using System;
using System.Collections.Generic;
using Realms;

namespace MyProject.Models
{
    public class GamePhoto : RealmObject
    {
        [PrimaryKey]
        public long PhotoId { get; set; }

        public DateTimeOffset TakenAt { get; set; }

        public DateTimeOffset PublishedAt { get; set; }

        public GameUser Publisher { get; set; }

        public GameLevel Level { get; set; }

        public string LevelName { get; set; }

        public string LevelType { get; set; }

        public long LevelId { get; set; }

        public GameAsset SmallAsset { get; set; }

        public GameAsset MediumAsset { get; set; }

        public GameAsset LargeAsset { get; set; }

        public string PlanHash { get; set; }

        public GameUser Subject1User { get; set; }

        public string Subject1DisplayName { get; set; }

        public IList<float> Subject1Bounds { get; }

        public GameUser Subject2User { get; set; }

        public string Subject2DisplayName { get; set; }

        public IList<float> Subject2Bounds { get; }

        public GameUser Subject3User { get; set; }

        public string Subject3DisplayName { get; set; }

        public IList<float> Subject3Bounds { get; }

        public GameUser Subject4User { get; set; }

        public string Subject4DisplayName { get; set; }

        public IList<float> Subject4Bounds { get; }
    }
}
