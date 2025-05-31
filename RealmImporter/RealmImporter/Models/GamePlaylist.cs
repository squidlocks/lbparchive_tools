// Please note : [Backlink] properties and default values are not represented
// in the schema and thus will not be part of the generated models

using System;
using System.Collections.Generic;
using Realms;

namespace MyProject.Models
{
    public class GamePlaylist : RealmObject
    {
        [PrimaryKey]
        public long PlaylistId { get; set; }

        public GameUser Publisher { get; set; }

        [Required]
        public string Name { get; set; }

        [Required]
        public string Description { get; set; }

        [Required]
        public string IconHash { get; set; }

        public long LocationX { get; set; }

        public long LocationY { get; set; }

        public DateTimeOffset CreationDate { get; set; }

        public DateTimeOffset LastUpdateDate { get; set; }

        public bool IsRoot { get; set; }
    }
}
