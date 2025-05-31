// Please note : [Backlink] properties and default values are not represented
// in the schema and thus will not be part of the generated models

using System;
using System.Collections.Generic;
using Realms;

namespace MyProject.Models
{
    public class GameLevelComment : RealmObject
    {
        [PrimaryKey]
        public long SequentialId { get; set; }

        public GameUser Author { get; set; }

        public GameLevel Level { get; set; }

        [Required]
        public string Content { get; set; }

        public DateTimeOffset Timestamp { get; set; }
    }
}
