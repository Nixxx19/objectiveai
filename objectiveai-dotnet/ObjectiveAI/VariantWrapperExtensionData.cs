// Adds [JsonExtensionData] to all variant wrapper types so that unknown fields
// are captured during deserialization and re-emitted during serialization.
// This is needed because variant wrappers only declare discriminator properties
// (e.g. "type") but the underlying Rust types have additional fields (e.g. "depth").
// Without this, JSON round-trips through these types lose data.

using System.Text.Json;
using System.Text.Json.Serialization;

// --- Functions.TaskExpression wrappers ---
namespace ObjectiveAI.Functions
{
    public partial class TaskExpressionScalarFunction
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class TaskExpressionVectorFunction
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class TaskExpressionVectorCompletion
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class TaskExpressionPlaceholderScalarFunction
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class TaskExpressionPlaceholderVectorFunction
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    // --- Functions.Task wrappers ---
    public partial class TaskScalarFunction
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class TaskVectorFunction
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class TaskVectorCompletion
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class TaskPlaceholderScalarFunction
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class TaskPlaceholderVectorFunction
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }
}

// --- Functions.Inventions.State wrappers ---
namespace ObjectiveAI.Functions.Inventions.State
{
    // State variants
    public partial class StateAlphaScalarBranch
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class StateAlphaScalarLeaf
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class StateAlphaVectorBranch
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class StateAlphaVectorLeaf
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    // ParamsState variants
    public partial class ParamsStateAlphaScalarBranch
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class ParamsStateAlphaScalarLeaf
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class ParamsStateAlphaVectorBranch
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class ParamsStateAlphaVectorLeaf
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class ParamsStateAlphaScalar
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class ParamsStateAlphaVector
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }
}

// --- Functions.AlphaScalar.BranchTaskExpression wrappers ---
namespace ObjectiveAI.Functions.AlphaScalar
{
    public partial class BranchTaskExpressionScalarFunction
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class BranchTaskExpressionPlaceholderScalarFunction
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }
}

// --- Functions.AlphaVector.BranchTaskExpression wrappers ---
namespace ObjectiveAI.Functions.AlphaVector
{
    public partial class BranchTaskExpressionScalarFunction
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class BranchTaskExpressionVectorFunction
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class BranchTaskExpressionPlaceholderScalarFunction
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class BranchTaskExpressionPlaceholderVectorFunction
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    // PartialPlaceholderBranchTaskExpression wrappers
    public partial class PartialPlaceholderBranchTaskExpressionPlaceholderScalarFunction
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class PartialPlaceholderBranchTaskExpressionPlaceholderVectorFunction
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }
}

// --- Agent.Completions.Message wrappers ---
namespace ObjectiveAI.Agent.Completions.Message
{
    // Message variants
    public partial class MessageDeveloper
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class MessageSystem
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class MessageUser
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class MessageAssistant
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class MessageTool
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    // MessageExpression variants
    public partial class MessageExpressionDeveloper
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class MessageExpressionSystem
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class MessageExpressionUser
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class MessageExpressionAssistant
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }

    public partial class MessageExpressionTool
    {
        [JsonExtensionData]
        public Dictionary<string, JsonElement>? ExtensionData { get; set; }
    }
}
