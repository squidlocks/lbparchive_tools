// Please note : [Backlink] properties and default values are not represented
// in the schema and thus will not be part of the generated models

using System;
using System.Collections.Generic;
using Realms;
using MongoDB.Bson;

namespace MyProject.Models
{
    public class Token : RealmObject
    {
        [PrimaryKey]
        public ObjectId TokenId { get; set; }

        public string TokenData { get; set; }

        public long _TokenType { get; set; }

        public long _TokenPlatform { get; set; }

        public long _TokenGame { get; set; }

        public DateTimeOffset ExpiresAt { get; set; }

        public DateTimeOffset LoginDate { get; set; }

        public string IpAddress { get; set; }

        public GameUser User { get; set; }

        public string Digest { get; set; }

        public bool IsHmacDigest { get; set; }
    }
}
