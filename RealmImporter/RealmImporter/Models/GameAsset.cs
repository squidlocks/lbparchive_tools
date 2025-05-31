// Please note : [Backlink] properties and default values are not represented
// in the schema and thus will not be part of the generated models

using System;
using System.Collections.Generic;
using Realms;

namespace MyProject.Models
{
    public class GameAsset : RealmObject
    {
        [PrimaryKey]
        [Required]
        public string AssetHash { get; set; }

        public GameUser OriginalUploader { get; set; }

        public DateTimeOffset UploadDate { get; set; }

        public bool IsPSP { get; set; }

        public long SizeInBytes { get; set; }

        public long _AssetType { get; set; }

        public long _AssetSerializationMethod { get; set; }

        [Required]
        public IList<string> Dependencies { get; }

        public string AsMainlineIconHash { get; set; }

        public string AsMipIconHash { get; set; }

        public string AsMainlinePhotoHash { get; set; }
    }
}
