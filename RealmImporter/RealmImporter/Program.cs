
// Program.cs
using System;
using System.IO;
using System.Linq;
using System.Collections.Generic;
using System.Data.SQLite;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Newtonsoft.Json.Serialization;
using MongoDB.Bson;
using Realms;
using MyProject.Models;
#nullable enable

namespace RealmImporter
{
    class ImportData
    {
        public List<GameUser> Users { get; set; } = new();
        public List<GameLevel> Levels { get; set; } = new();
        public List<AssetDependencyRelation> Relations { get; set; } = new();
        public List<GameAsset> Assets { get; set; } = new();
    }

    class ObjectIdJsonConverter : JsonConverter<ObjectId>
    {
        public override ObjectId ReadJson(JsonReader reader, Type objectType,
                                          ObjectId existingValue, bool hasExistingValue,
                                          JsonSerializer serializer)
        {
            if (reader.TokenType == JsonToken.String &&
                ObjectId.TryParse((string)reader.Value, out var sId))
                return sId;

            if (reader.TokenType == JsonToken.StartObject)
            {
                var jo = JObject.Load(reader);
                var oid = jo["$oid"]?.Value<string>();
                if (oid != null && ObjectId.TryParse(oid, out var oId))
                    return oId;
            }

            return ObjectId.GenerateNewId();
        }

        public override void WriteJson(JsonWriter writer, ObjectId value,
                                       JsonSerializer serializer)
            => writer.WriteValue(value.ToString());
    }

    class Program
    {
        static void PrintUsage()
        {
            Console.Error.WriteLine("Usage: RealmImporter.exe <template.realm> <output.realm> [seed]");
            Console.Error.WriteLine("  seed   – if present, after importing JSON it'll also seed unique-play relations");
            Console.Error.WriteLine();
            Console.Error.WriteLine("Note: the SQLite DB file must be named `dry.db` and sit in the working directory.");
        }

        static int Main(string[] args)
        {
            if (args.Length < 2 || args.Length > 3)
            {
                PrintUsage();
                return 1;
            }

            // always look for dry.db in CWD
            var dbPath = Path.Combine(Directory.GetCurrentDirectory(), "dry.db");
            var template = Path.GetFullPath(args[0]);
            var output = Path.GetFullPath(args[1]);
            bool doSeed = args.Length == 3 &&
                           args[2].Equals("seed", StringComparison.OrdinalIgnoreCase);

            Console.WriteLine($"🔧 CWD: {Directory.GetCurrentDirectory()}");
            Console.WriteLine($"🗄️  SQLite DB → {dbPath}");
            Console.WriteLine($"📄 Template → {template}");
            Console.WriteLine($"📄 Output   → {output}");
            if (doSeed)
                Console.WriteLine("🌱  Will seed by unique-play count after import.");

            if (!File.Exists(dbPath))
            {
                Console.Error.WriteLine($"❌ Could not find `dry.db` in {Directory.GetCurrentDirectory()}");
                return 1;
            }

            if (!File.Exists(output))
            {
                try
                {
                    File.Copy(template, output);
                    Console.WriteLine("✳️  Created new Realm from template");
                }
                catch (Exception ex)
                {
                    Console.Error.WriteLine($"Failed to copy template → {ex.Message}");
                    return 1;
                }
            }
            else
            {
                Console.WriteLine("► Output Realm exists—appending to it");
            }

            // 1) import.json → DTO
            ImportData import;
            try
            {
                var json = File.ReadAllText("import.json");
                var settings = new JsonSerializerSettings
                {
                    Converters = new List<JsonConverter> { new ObjectIdJsonConverter() },
                    MissingMemberHandling = MissingMemberHandling.Ignore,
                    NullValueHandling = NullValueHandling.Include,
                    ContractResolver = new DefaultContractResolver
                    {
                        NamingStrategy = new CamelCaseNamingStrategy()
                    }
                };
                import = JsonConvert.DeserializeObject<ImportData>(json, settings)
                         ?? new ImportData();
            }
            catch (Exception ex)
            {
                Console.Error.WriteLine($"Error reading import.json → {ex.Message}");
                return 1;
            }

            foreach (var u in import.Users)
            {
                var cfg = new RealmConfiguration(output)
                {
                    SchemaVersion = 161
                }
                ;
                using var realm = Realm.GetInstance(cfg);
                // Try to find an existing user in the Realm with the same Username:
                var existing = realm.All<GameUser>()
                                        .FirstOrDefault(x => x.Username == u.Username);
                if (existing != null)
                {
                    // reuse their primary key so update: true will hit this record
                    u.UserId = existing.UserId;
                }
                else if (u.UserId == ObjectId.Empty)
                {
                    // brand-new user
                    u.UserId = ObjectId.GenerateNewId();
                }

                if (string.IsNullOrWhiteSpace(u.Username))
                {
                    Console.ForegroundColor = ConsoleColor.Yellow;
                    Console.WriteLine($"⚠️  Blank Username on {u.UserId} → faking…");
                    Console.ResetColor();
                    u.Username = $"user_{u.UserId}";
                }

                // coalesce required strings
                u.EmailAddress ??= "";
                u.PasswordBcrypt ??= "";
                u.IconHash ??= "";
                u.PspIconHash ??= "";
                u.VitaIconHash ??= "";
                u.BetaIconHash ??= "";
                u.Description ??= "";
                u.BetaPlanetsHash ??= "";
                u.Lbp2PlanetsHash ??= "";
                u.Lbp3PlanetsHash ??= "";
                u.VitaPlanetsHash ??= "";
                u.YayFaceHash ??= "";
                u.BooFaceHash ??= "";
                u.MehFaceHash ??= "";
                u.PresenceServerAuthToken ??= "";
            }

            var fallbackUser = import.Users.FirstOrDefault();
            if (fallbackUser == null)
            {
                Console.Error.WriteLine("No users to import → nothing to link levels/assets to.");
                return 1;
            }

            // sanitize levels
            foreach (var lvl in import.Levels)
            {
                lvl.Title ??= "";
                lvl.IconHash ??= "";
                lvl.Description ??= "";
                lvl.OriginalPublisher ??= "";
                lvl.Publisher = fallbackUser;
            }

            // sanitize assets
            // sanitize assets
            // build a set of all icon‐hashes from the levels we just imported
            var iconHashes = new HashSet<string>(
            import.Levels
                                  .Select(l => l.IconHash)
                                  .Where(h => !string.IsNullOrEmpty(h))
                        );

            foreach (var asset in import.Assets)
            {
                // if this asset’s hash matches one of our level‐icon hashes,
                // then use it as its own AsMainlineIconHash
                if (iconHashes.Contains(asset.AssetHash))
                {
                    asset.AsMainlineIconHash = asset.AssetHash;
                }
                else
                {
                    asset.AsMainlineIconHash ??= "";
                }

                // leave the other icon fields untouched or defaulted
                asset.AsMipIconHash ??= "";
                asset.AsMainlinePhotoHash ??= "";
                asset.OriginalUploader = fallbackUser;
            }
            // write into realm
            try
            {
                var cfg = new RealmConfiguration(output)
                {
                    SchemaVersion = 161
                }
                ;
                using var realm = Realm.GetInstance(cfg);
                using var tx = realm.BeginWrite();

                Console.WriteLine($"⛓️  Opening Realm at: {cfg.DatabasePath}");
                Console.WriteLine($"[DEBUG] BEFORE COMMIT ␦ Users={realm.All<GameUser>().Count()}, Levels={realm.All<GameLevel>().Count()}, Relations={realm.All<AssetDependencyRelation>().Count()}, Assets={realm.All<GameAsset>().Count()}");

                foreach (var u in import.Users) realm.Add(u, update: true);
                foreach (var l in import.Levels) realm.Add(l, update: true);
                foreach (var r in import.Relations)
                {
                    bool exists = realm.All<AssetDependencyRelation>()
                                       .Any(x => x.Dependent == r.Dependent
                                              && x.Dependency == r.Dependency);
                    if (!exists)
                    {
                        realm.Add(r, update: true);
                    }
                }
                foreach (var a in import.Assets) realm.Add(a, update: true);

                tx.Commit();
                Console.WriteLine($"[DEBUG] AFTER COMMIT  ␦ Users={realm.All<GameUser>().Count()}, Levels={realm.All<GameLevel>().Count()}, Relations={realm.All<AssetDependencyRelation>().Count()}, Assets={realm.All<GameAsset>().Count()}");
            }
            catch (Exception ex)
            {
                Console.Error.WriteLine($"Error writing to realm → {ex.Message}");
                return 1;
            }

            Console.WriteLine(
    $"✅ Imported {import.Users.Count} users, " +
    $"{import.Levels.Count} levels, " +
    $"{import.Relations.Count} relations, " +
    $"{import.Assets.Count} assets → {output}"
);
            // Console.WriteLine("✅ JSON import complete.");

            // 3) optionally seed by uniquePlayCount
            if (doSeed)
            {
                var config = new RealmConfiguration(output)
                {
                    SchemaVersion = 161
                };
                using var realm = Realm.GetInstance(config);
                SeedByUniquePlayCount(dbPath, realm);
                SeedCreatorHearts(dbPath, realm);
                SeedLevelHearts(dbPath, realm);
            }

            return 0;
        }

        /// <summary>
        /// Reads slot.uniquePlayCount from dry.db,
        /// allocates exactly the max unique count of dummy users,
        /// and for each level seeds one UniquePlayLevelRelation + one PlayLevelRelation(Count=1)
        /// per dummy user up to that level’s uniquePlayCount.
        /// </summary>
        static void SeedByUniquePlayCount(string sqliteDbPath, Realm realm)
        {
            // load uniquePlayCount
            var counts = new Dictionary<long, long>();
            using (var conn = new SQLiteConnection($"Data Source={sqliteDbPath}"))
            {
                conn.Open();
                using var cmd = new SQLiteCommand("SELECT id, uniquePlayCount FROM slot", conn);
                using var rdr = cmd.ExecuteReader();
                while (rdr.Read())
                {
                    var id = rdr.GetInt64(0);
                    var uniq = rdr.IsDBNull(1) ? 0L : rdr.GetInt64(1);
                    counts[id] = uniq;
                }
            }

            // determine max needed
            int maxUnique = counts.Values.DefaultIfEmpty(0L).Max() switch
            {
                long v when v > int.MaxValue => int.MaxValue,
                long v => (int)v
            };
            Console.WriteLine($"🔄 Creating {maxUnique:N0} dummy users…");

            // create dummy users
            var dummyUsers = new List<GameUser>(maxUnique);
            var now = DateTimeOffset.UtcNow;
            realm.Write(() =>
            {
                for (int i = 0; i < maxUnique; i++)
                {
                    var du = new GameUser
                    {
                        UserId = ObjectId.GenerateNewId(),
                        Username = $"dummy_user_{i}",
                        EmailAddress = "",
                        PasswordBcrypt = "",
                        IconHash = "",
                        PspIconHash = "",
                        VitaIconHash = "",
                        BetaIconHash = "",
                        Description = "",
                        BetaPlanetsHash = "",
                        Lbp2PlanetsHash = "",
                        Lbp3PlanetsHash = "",
                        VitaPlanetsHash = "",
                        YayFaceHash = "",
                        BooFaceHash = "",
                        MehFaceHash = "",
                        PresenceServerAuthToken = ""
                    };
                    realm.Add(du, update: false);
                    dummyUsers.Add(du);
                }
            });
            Console.WriteLine("✅ Dummy users created.");

            // seed per-level
            realm.Write(() =>
            {
                foreach (var lvl in realm.All<GameLevel>())
                {
                    if (!counts.TryGetValue(lvl.LevelId, out var c) || c <= 0) continue;
                    Console.WriteLine($"→ \"{lvl.Title}\" → {c:N0} unique plays");
                    for (int u = 0; u < c; u++)
                    {
                        var user = dummyUsers[u];
                        realm.Add(new UniquePlayLevelRelation
                        {
                            Level = lvl,
                            User = user,
                            Timestamp = now
                        }, update: false);
                        realm.Add(new PlayLevelRelation
                        {
                            Level = lvl,
                            User = user,
                            Timestamp = now,
                            Count = 1
                        }, update: false);
                    }
                }
            });

            Console.WriteLine("✅ Seeding by uniquePlayCount complete.");
        }


        static void SeedCreatorHearts(string sqliteDbPath, Realm realm)
        {
            // 1) load creator heart counts keyed by npHandle (= Username)
            var creatorHearts = new Dictionary<string, long>();
            using (var conn = new SQLiteConnection($"Data Source={sqliteDbPath}"))
            {
                conn.Open();
                using var cmd = new SQLiteCommand("SELECT npHandle, heartCount FROM \"user\"", conn);
                using var rdr = cmd.ExecuteReader();
                while (rdr.Read())
                {
                    var name = rdr.GetString(0);
                    var hc = rdr.IsDBNull(1) ? 0L : rdr.GetInt64(1);
                    creatorHearts[name] = hc;
                }
            }

            // 2) grab your dummy users in a stable order
            var dummyUsers = realm.All<GameUser>()
                                  .Where(u => u.Username.StartsWith("dummy_user_"))
                                  .OrderBy(u => u.Username)
                                  .ToList();

            // 3) for each real user, add one FavouriteUserRelation per heart
            realm.Write(() =>
            {
                foreach (var target in realm.All<GameUser>())
                {
                    if (!creatorHearts.TryGetValue(target.Username, out var count) || count <= 0)
                        continue;

                    Console.WriteLine($"→ Giving {count:N0} hearts to creator “{target.Username}”");
                    var toSeed = (int)Math.Min(count, dummyUsers.Count);
                    for (int i = 0; i < toSeed; i++)
                    {
                        realm.Add(new FavouriteUserRelation
                        {
                            UserFavouriting = dummyUsers[i],
                            UserToFavourite = target
                        }, update: false);
                    }
                }
            });

            Console.WriteLine("✅ Creator hearts seeded.");
        }


        static void SeedLevelHearts(string sqliteDbPath, Realm realm)
        {
            // 1) load level heart counts keyed by slot.id
            var levelHearts = new Dictionary<long, long>();
            using (var conn = new SQLiteConnection($"Data Source={sqliteDbPath}"))
            {
                conn.Open();
                using var cmd = new SQLiteCommand("SELECT id, heartCount FROM slot", conn);
                using var rdr = cmd.ExecuteReader();
                while (rdr.Read())
                {
                    var id = rdr.GetInt64(0);
                    var hc = rdr.IsDBNull(1) ? 0L : rdr.GetInt64(1);
                    levelHearts[id] = hc;
                }
            }

            // 2) grab your dummy users again
            var dummyUsers = realm.All<GameUser>()
                                  .Where(u => u.Username.StartsWith("dummy_user_"))
                                  .OrderBy(u => u.Username)
                                  .ToList();

            // 3) for each GameLevel, add one FavouriteLevelRelation per heart
            realm.Write(() =>
            {
                foreach (var lvl in realm.All<GameLevel>())
                {
                    if (!levelHearts.TryGetValue(lvl.LevelId, out var count) || count <= 0)
                        continue;

                    Console.WriteLine($"→ Giving {count:N0} hearts to level “{lvl.Title}”");
                    var toSeed = (int)Math.Min(count, dummyUsers.Count);
                    for (int i = 0; i < toSeed; i++)
                    {
                        realm.Add(new FavouriteLevelRelation
                        {
                            User = dummyUsers[i],
                            Level = lvl
                        }, update: false);
                    }
                }
            });

            Console.WriteLine("✅ Level hearts seeded.");
        }
    }
}
