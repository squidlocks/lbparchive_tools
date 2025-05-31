// Please note : [Backlink] properties and default values are not represented
// in the schema and thus will not be part of the generated models

using System;
using System.Collections.Generic;
using Realms;

namespace MyProject.Models
{
    public class RequestStatistics : RealmObject
    {
        public long TotalRequests { get; set; }

        public long ApiRequests { get; set; }

        public long GameRequests { get; set; }
    }
}
