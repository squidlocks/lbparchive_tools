// Please note : [Backlink] properties and default values are not represented
// in the schema and thus will not be part of the generated models

using System;
using System.Collections.Generic;
using Realms;
using MongoDB.Bson;

namespace MyProject.Models
{
    public class LevelCommentRelation : RealmObject
    {
        public ObjectId CommentRelationId { get; set; }

        public GameUser User { get; set; }

        public GameLevelComment Comment { get; set; }

        public long _RatingType { get; set; }

        public DateTimeOffset Timestamp { get; set; }
    }
}
