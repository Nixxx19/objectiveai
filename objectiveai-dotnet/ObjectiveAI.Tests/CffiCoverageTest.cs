/*
 * CFFI coverage test: verifies every extern "C" function in
 * objectiveai-rs-cffi/src/lib.rs has a matching public static method in Cffi.cs.
 *
 * Matches Go's cffi_coverage_test.go pattern.
 */

using System.Reflection;
using System.Text.RegularExpressions;

namespace ObjectiveAI.Tests;

public class CffiCoverageTest
{
    [Fact]
    public void CffiCoverage()
    {
        // Locate repo root
        var baseDir = AppContext.BaseDirectory;
        var repoRoot = Path.GetFullPath(Path.Combine(baseDir, "..", "..", "..", "..", ".."));

        // Read Rust lib.rs
        var rustPath = Path.Combine(repoRoot, "objectiveai-rs-cffi", "src", "lib.rs");
        Assert.True(File.Exists(rustPath), $"Rust lib.rs not found: {rustPath}");
        var rustSrc = File.ReadAllText(rustPath);

        // Extract all extern "C" function names (objectiveai_*)
        var rustFnRe = new Regex(@"pub\s+(?:unsafe\s+)?extern\s+""C""\s+fn\s+(objectiveai_(\w+))");
        var rustMatches = rustFnRe.Matches(rustSrc);
        Assert.True(rustMatches.Count > 0, "Found no extern \"C\" functions in lib.rs");

        var rustFns = new List<(string Full, string Stripped)>();
        foreach (Match m in rustMatches)
        {
            rustFns.Add((m.Groups[1].Value, m.Groups[2].Value));
        }

        // Get all public static methods from Cffi class
        var cffiType = typeof(Cffi);
        var csharpMethods = cffiType
            .GetMethods(BindingFlags.Public | BindingFlags.Static)
            .Where(m => m.DeclaringType == typeof(Cffi))
            .Select(m => m.Name)
            .ToHashSet();

        // Build expected C# method names from Rust (snake_case → PascalCase)
        var expectedMethods = new Dictionary<string, string>(); // PascalName → RustName
        foreach (var (full, stripped) in rustFns)
        {
            var pascalName = SnakeToPascal(stripped);
            expectedMethods[pascalName] = full;
        }

        // Assert every Rust function has a C# wrapper
        var missing = new List<string>();
        foreach (var (pascal, rust) in expectedMethods)
        {
            if (!csharpMethods.Contains(pascal))
                missing.Add($"{rust} -> {pascal}");
        }

        if (missing.Count > 0)
        {
            Assert.Fail($"Cffi.cs is missing {missing.Count} function(s):\n  {string.Join("\n  ", missing)}");
        }

        // Assert no extra C# methods beyond what Rust declares
        var extra = csharpMethods
            .Where(m => !expectedMethods.ContainsKey(m))
            .ToList();

        if (extra.Count > 0)
        {
            Assert.Fail($"Cffi.cs has {extra.Count} unexpected method(s):\n  {string.Join("\n  ", extra)}");
        }
    }

    private static string SnakeToPascal(string s)
    {
        var sb = new System.Text.StringBuilder();
        bool upper = true;
        foreach (var c in s)
        {
            if (c == '_')
            {
                upper = true;
                continue;
            }
            sb.Append(upper ? char.ToUpper(c) : c);
            upper = false;
        }
        return sb.ToString();
    }
}
