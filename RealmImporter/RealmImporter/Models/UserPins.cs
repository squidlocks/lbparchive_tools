// Please note : [Backlink] properties and default values are not represented
// in the schema and thus will not be part of the generated models

using System;
using System.Collections.Generic;
using Realms;

namespace MyProject.Models
{
    public class UserPins : EmbeddedObject
    {
        public IList<long> Progress { get; }

        public IList<long> Awards { get; }

        public IList<long> ProfilePins { get; }
    }
}
