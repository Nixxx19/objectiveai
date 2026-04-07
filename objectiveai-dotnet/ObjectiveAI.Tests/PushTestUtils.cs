/*
 * Test utilities for push fuzz tests.
 * Matches Go's push_test_utils_test.go, JS's mergeTestUtil.ts,
 * and Python's push_test_utils.py.
 */

using System.Text.Json;

namespace ObjectiveAI.Tests;

public static class PushTestUtils
{
    private static readonly JsonSerializerOptions JsonOpts = new();

    /// <summary>
    /// Deep copy via JSON round-trip (same as Go's deepCopy).
    /// </summary>
    public static T DeepCopy<T>(T value)
    {
        var json = JsonSerializer.Serialize(value, JsonOpts);
        return JsonSerializer.Deserialize<T>(json, JsonOpts)!;
    }

    /// <summary>
    /// Serialize to a map (Dictionary) via JSON round-trip (same as Go's toMap).
    /// </summary>
    public static Dictionary<string, object?> ToMap<T>(T value)
    {
        var json = JsonSerializer.Serialize(value, JsonOpts);
        return JsonSerializer.Deserialize<Dictionary<string, object?>>(json)!;
    }

    /// <summary>
    /// Recursively round all floating-point numbers to 8 significant figures.
    /// Double-rounds through 12 digits first to normalize 1-ULP representation artifacts.
    /// Matches Go's rounded(), JS's mergeTestUtil.ts, and Python's push_test_utils.py.
    /// </summary>
    public static object? Rounded(object? value)
    {
        if (value is JsonElement el)
            return Rounded(JsonElementToObject(el));

        if (value is double d)
        {
            if (d == 0 || double.IsInfinity(d) || double.IsNaN(d))
                return d;
            // Double-round: first to 12 sig figs, then to 8
            var s12 = d.ToString("G12");
            var f12 = double.Parse(s12);
            var s8 = f12.ToString("G8");
            var f8 = double.Parse(s8);
            return f8;
        }

        if (value is float f)
            return Rounded((double)f);

        if (value is Dictionary<string, object?> dict)
        {
            var result = new Dictionary<string, object?>();
            foreach (var (k, v) in dict)
                result[k] = Rounded(v);
            return result;
        }

        if (value is List<object?> list)
            return list.Select(Rounded).ToList();

        return value;
    }

    /// <summary>
    /// Assert that two values are equal after rounding floats.
    /// </summary>
    public static void AssertRoundedEqual(string label, object? got, object? want)
    {
        var gotRounded = Rounded(got);
        var wantRounded = Rounded(want);

        var gotJson = JsonSerializer.Serialize(gotRounded, new JsonSerializerOptions { WriteIndented = true });
        var wantJson = JsonSerializer.Serialize(wantRounded, new JsonSerializerOptions { WriteIndented = true });

        if (gotJson != wantJson)
        {
            throw new Xunit.Sdk.XunitException(
                $"Push mismatch at {label}:\n\n--- C# Push ---\n{gotJson}\n\n--- CFFI Merge ---\n{wantJson}");
        }
    }

    private static object? JsonElementToObject(JsonElement el)
    {
        return el.ValueKind switch
        {
            JsonValueKind.Object => el.EnumerateObject().ToDictionary(p => p.Name, p => JsonElementToObject(p.Value)),
            JsonValueKind.Array => el.EnumerateArray().Select(JsonElementToObject).ToList<object?>(),
            JsonValueKind.String => el.GetString(),
            JsonValueKind.Number => el.GetDouble(),
            JsonValueKind.True => true,
            JsonValueKind.False => false,
            JsonValueKind.Null => null,
            _ => null,
        };
    }
}
