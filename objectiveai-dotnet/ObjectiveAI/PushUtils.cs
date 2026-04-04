namespace ObjectiveAI;

/// <summary>
/// Static push helper methods for streaming chunk accumulation.
/// Matches Go's push_utils.go and Python's push_utils.py.
/// </summary>
public static class PushUtils
{
    /// <summary>
    /// Merge items by matching a required index field.
    /// Items with matching index are merged via push; new items are appended.
    /// </summary>
    public static void PushByIndex<T>(List<T> self, List<T>? other, Func<T, long> getIndex, Action<T, T> push)
    {
        if (other == null || other.Count == 0) return;
        var indexMap = new Dictionary<long, int>();
        for (int i = 0; i < self.Count; i++)
            indexMap[getIndex(self[i])] = i;

        foreach (var item in other)
        {
            var idx = getIndex(item);
            if (indexMap.TryGetValue(idx, out var pos))
            {
                push(self[pos], item);
            }
            else
            {
                self.Add(item);
                indexMap[idx] = self.Count - 1;
            }
        }
    }

    /// <summary>
    /// Merge items by matching a nullable index field.
    /// Items with null index are always appended (never merged).
    /// </summary>
    public static void PushByNullableIndex<T>(List<T> self, List<T>? other, Func<T, long?> getIndex, Action<T, T> push)
    {
        if (other == null || other.Count == 0) return;
        var indexMap = new Dictionary<long, int>();
        for (int i = 0; i < self.Count; i++)
        {
            var idx = getIndex(self[i]);
            if (idx.HasValue)
                indexMap[idx.Value] = i;
        }

        foreach (var item in other)
        {
            var idx = getIndex(item);
            if (idx.HasValue && indexMap.TryGetValue(idx.Value, out var pos))
            {
                push(self[pos], item);
            }
            else
            {
                self.Add(item);
                if (idx.HasValue)
                    indexMap[idx.Value] = self.Count - 1;
            }
        }
    }

    /// <summary>Latest non-null value wins.</summary>
    public static T? PushReplace<T>(T? self, T? other) where T : class
        => other ?? self;

    /// <summary>Latest non-null value wins (value types).</summary>
    public static T? PushReplaceValue<T>(T? self, T? other) where T : struct
        => other ?? self;

    /// <summary>Concatenate optional strings.</summary>
    public static string? PushOptionString(string? self, string? other)
    {
        if (self != null && other != null) return self + other;
        return other ?? self;
    }

    /// <summary>Sum optional longs.</summary>
    public static long? PushOptionLong(long? self, long? other)
    {
        if (self.HasValue && other.HasValue) return self.Value + other.Value;
        return other ?? self;
    }

    /// <summary>Sum optional ulongs.</summary>
    public static ulong? PushOptionUlong(ulong? self, ulong? other)
    {
        if (self.HasValue && other.HasValue) return self.Value + other.Value;
        return other ?? self;
    }

    /// <summary>Sum optional doubles.</summary>
    public static double? PushOptionDouble(double? self, double? other)
    {
        if (self.HasValue && other.HasValue) return self.Value + other.Value;
        return other ?? self;
    }

    /// <summary>Monotonic boolean: once true, stays true.</summary>
    public static bool? PushLazySetTrue(bool? self, bool? other)
    {
        if (other.HasValue && other.Value) return true;
        return self;
    }

    /// <summary>
    /// Conditional merge of optional sub-objects.
    /// Both present → push; only other → adopt; otherwise no change.
    /// </summary>
    public static void PushOption<T>(ref T? self, T? other, Action<T, T> push) where T : class
    {
        if (self != null && other != null)
            push(self, other);
        else if (other != null)
            self = other;
    }
}
