// Please note : [Backlink] properties and default values are not represented
// in the schema and thus will not be part of the generated models

using System;
using System.Collections.Generic;
using Realms;

namespace MyProject.Models
{
    public class GameSkillReward : EmbeddedObject
    {
        public long Id { get; set; }

        public bool Enabled { get; set; }

        public string Title { get; set; }

        public float RequiredAmount { get; set; }

        public long _ConditionType { get; set; }
    }
}
