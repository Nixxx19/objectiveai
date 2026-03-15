/*
 * Strict roundtrip test harness for JSON Schema validation.
 *
 * THIS FILE MUST NEVER BE MODIFIED.
 *
 * This harness is purposefully strict. It loads the original JSON schemas from
 * objectiveai-json-schema/ exactly as they are on disk — no normalization, no
 * massaging, no xfail. The original schema is treated as the canonical source
 * of truth and is never altered.
 *
 * The contract is simple: the caller passes a schema title and a dictionary.
 * This harness loads the original, serializes both sides using the canonical
 * key ordering from the JSON schema builder (objectiveai-json-schema/builder/),
 * and compares the serialized strings for exact equality.
 *
 * Key ordering rules (matching the Rust builder):
 *   - Inside "properties": keys are sorted alphabetically.
 *   - Outside "properties": keys are sorted by KeywordOrder, with any
 *     unknown keys placed at the end.
 *
 * If a test fails, the fix belongs in the caller's conversion/normalization
 * logic or in the code generator — never in this file.
 */

using System.Text.Json;

namespace ObjectiveAI.Tests;

public static class RoundtripHarness
{
    // Canonical key ordering for JSON Schema keywords.
    // Matches KEYWORD_ORDER in objectiveai-json-schema/builder/src/main.rs.
    public static readonly string[] KeywordOrder =
    [
        "title",
        "description",
        "type",
        "enum",
        "anyOf",
        "$ref",
        "properties",
        "additionalProperties",
        "items",
        "minItems",
        "maxItems",
        "minimum",
        "maximum",
        "pattern",
        "format",
        "default",
    ];

    private static readonly Dictionary<string, int> KeywordRank =
        KeywordOrder.Select((kw, i) => (kw, i)).ToDictionary(x => x.kw, x => x.i);

    private static readonly int UnknownRank = KeywordOrder.Length;

    private static readonly Lazy<Dictionary<string, JsonElement>> OriginalSchemas = new(LoadOriginalSchemas);

    public static IReadOnlyDictionary<string, JsonElement> Schemas => OriginalSchemas.Value;

    public static IReadOnlySet<string> AllTitles => OriginalSchemas.Value.Keys.ToHashSet();

    private static Dictionary<string, JsonElement> LoadOriginalSchemas()
    {
        // Navigate from the test binary to the repo root
        var baseDir = AppContext.BaseDirectory;
        var repoRoot = Path.GetFullPath(Path.Combine(baseDir, "..", "..", "..", "..", ".."));
        var schemaDir = Path.Combine(repoRoot, "objectiveai-json-schema");

        if (!Directory.Exists(schemaDir))
            throw new DirectoryNotFoundException($"Schema directory not found: {schemaDir}");

        var schemas = new Dictionary<string, JsonElement>();
        foreach (var file in Directory.GetFiles(schemaDir, "*.json").OrderBy(f => f))
        {
            var doc = JsonDocument.Parse(File.ReadAllText(file));
            var root = doc.RootElement;
            if (root.TryGetProperty("title", out var titleEl))
            {
                schemas[titleEl.GetString()!] = root.Clone();
            }
        }
        return schemas;
    }

    /// <summary>
    /// Assert that a converted schema exactly matches the original on disk.
    /// Both the original and <paramref name="converted"/> are serialized using
    /// the canonical key ordering before comparison.
    /// </summary>
    public static void AssertSchemaMatches(string title, Dictionary<string, object?> converted)
    {
        var original = OriginalSchemas.Value[title];
        var expectedStr = Serialize(JsonElementToDict(original));
        var actualStr = Serialize(converted);

        if (actualStr != expectedStr)
        {
            throw new Xunit.Sdk.XunitException(
                $"Schema mismatch for '{title}':\n" +
                $"\n--- Expected (original from objectiveai-json-schema/) ---\n" +
                $"{expectedStr}\n" +
                $"\n--- Got (C#-derived) ---\n" +
                $"{actualStr}");
        }
    }

    private static string Serialize(object? value)
    {
        var ordered = OrderKeys(value, insideProperties: false);
        return JsonSerializer.Serialize(ordered, new JsonSerializerOptions
        {
            WriteIndented = true,
            Encoder = System.Text.Encodings.Web.JavaScriptEncoder.UnsafeRelaxedJsonEscaping,
        });
    }

    private static object? OrderKeys(object? value, bool insideProperties)
    {
        if (value is Dictionary<string, object?> dict)
        {
            var recursed = dict.ToDictionary(
                kv => kv.Key,
                kv => OrderKeys(kv.Value, insideProperties: kv.Key == "properties")
            );

            IEnumerable<KeyValuePair<string, object?>> sorted;
            if (insideProperties)
            {
                sorted = recursed.OrderBy(kv => kv.Key, StringComparer.Ordinal);
            }
            else
            {
                sorted = recursed.OrderBy(kv =>
                    KeywordRank.TryGetValue(kv.Key, out var rank) ? rank : UnknownRank);
            }
            return new Dictionary<string, object?>(sorted);
        }

        if (value is List<object?> list)
        {
            return list.Select(v => OrderKeys(v, insideProperties: false)).ToList();
        }

        return value;
    }

    private static Dictionary<string, object?> JsonElementToDict(JsonElement el)
    {
        var dict = new Dictionary<string, object?>();
        foreach (var prop in el.EnumerateObject())
        {
            dict[prop.Name] = JsonElementToObject(prop.Value);
        }
        return dict;
    }

    private static object? JsonElementToObject(JsonElement el)
    {
        return el.ValueKind switch
        {
            JsonValueKind.Object => JsonElementToDict(el),
            JsonValueKind.Array => el.EnumerateArray().Select(JsonElementToObject).ToList<object?>(),
            JsonValueKind.String => el.GetString(),
            JsonValueKind.Number => ParseNumber(el),
            JsonValueKind.True => true,
            JsonValueKind.False => false,
            JsonValueKind.Null => null,
            _ => null,
        };
    }

    private static object ParseNumber(JsonElement el)
    {
        // Preserve the exact JSON representation for roundtrip
        var raw = el.GetRawText();
        if (raw.Contains('.') || raw.Contains('e') || raw.Contains('E'))
        {
            // Floating point
            return el.GetDouble();
        }
        // Integer — try long first, then ulong for very large values
        if (el.TryGetInt64(out var l))
            return l;
        if (el.TryGetUInt64(out var ul))
            return ul;
        return el.GetDouble();
    }
}
