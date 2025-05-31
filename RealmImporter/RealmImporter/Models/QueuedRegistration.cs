// Please note : [Backlink] properties and default values are not represented
// in the schema and thus will not be part of the generated models

using System;
using System.Collections.Generic;
using Realms;
using MongoDB.Bson;

namespace MyProject.Models
{
    public class QueuedRegistration : RealmObject
    {
        [PrimaryKey]
        public ObjectId RegistrationId { get; set; }

        [Indexed]
        public string Username { get; set; }

        public string EmailAddress { get; set; }

        [Indexed]
        public string PasswordBcrypt { get; set; }

        public DateTimeOffset ExpiryDate { get; set; }
    }
}
