// Please note : [Backlink] properties and default values are not represented
// in the schema and thus will not be part of the generated models

using System;
using System.Collections.Generic;
using Realms;
using MongoDB.Bson;

namespace MyProject.Models
{
    public class GameNotification : RealmObject
    {
        public ObjectId NotificationId { get; set; }

        public string Title { get; set; }

        public string Text { get; set; }

        public DateTimeOffset CreatedAt { get; set; }

        public GameUser User { get; set; }

        public string FontAwesomeIcon { get; set; }
    }
}
