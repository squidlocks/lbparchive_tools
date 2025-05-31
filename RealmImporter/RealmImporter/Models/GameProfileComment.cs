// Please note : [Backlink] properties and default values are not represented
// in the schema and thus will not be part of the generated models

using System;
using System.Collections.Generic;
using Realms;

namespace MyProject.Models
{
    public class GameProfileComment : RealmObject
    {
        [PrimaryKey]
        public long SequentialId { get; set; }

        public GameUser Author { get; set; }

        public GameUser Profile { get; set; }

        [Required]
        public string Content { get; set; }

        public DateTimeOffset Timestamp { get; set; }
    }
}
