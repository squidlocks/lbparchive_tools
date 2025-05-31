// Please note : [Backlink] properties and default values are not represented
// in the schema and thus will not be part of the generated models

using System;
using System.Collections.Generic;
using Realms;

namespace MyProject.Models
{
    public class GameReview : RealmObject
    {
        public long ReviewId { get; set; }

        public GameLevel Level { get; set; }

        public GameUser Publisher { get; set; }

        public DateTimeOffset PostedAt { get; set; }

        public string Labels { get; set; }

        public string Content { get; set; }
    }
}
