// Please note : [Backlink] properties and default values are not represented
// in the schema and thus will not be part of the generated models

using System;
using System.Collections.Generic;
using Realms;
using MongoDB.Bson;

namespace MyProject.Models
{
    public class Event : RealmObject
    {
        public ObjectId EventId { get; set; }

        public long _EventType { get; set; }

        public GameUser User { get; set; }

        public bool IsPrivate { get; set; }

        public DateTimeOffset Timestamp { get; set; }

        public long _StoredDataType { get; set; }

        public long? StoredSequentialId { get; set; }

        public ObjectId? StoredObjectId { get; set; }
    }
}
