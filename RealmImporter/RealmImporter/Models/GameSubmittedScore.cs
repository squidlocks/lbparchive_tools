// Please note : [Backlink] properties and default values are not represented
// in the schema and thus will not be part of the generated models

using System;
using System.Collections.Generic;
using Realms;
using MongoDB.Bson;

namespace MyProject.Models
{
    public class GameSubmittedScore : RealmObject
    {
        [PrimaryKey]
        public ObjectId ScoreId { get; set; }

        [Indexed]
        public long _Game { get; set; }

        public long _Platform { get; set; }

        public GameLevel Level { get; set; }

        public IList<GameUser> Players { get; }

        public DateTimeOffset ScoreSubmitted { get; set; }

        [Indexed]
        public long Score { get; set; }

        [Indexed]
        public long ScoreType { get; set; }
    }
}
