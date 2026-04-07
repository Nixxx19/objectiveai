# Mock Asset Renames

## Agents

| Old Name | New Name | New Description |
|----------|----------|-----------------|
| mock-agent-1 | schema-logprobs | Responds in JSON schema format with 3 logprobs for confidence calibration. |
| mock-agent-2 | instruction | Responds in natural language instruction format. |
| mock-agent-3 | tool-call | Responds via tool/function calling. |
| mock-agent-4 | instruction-logprobs | Responds in instruction format with 6 logprobs for fine-grained confidence. |

## Swarms

| Old Name | New Name | New Description |
|----------|----------|-----------------|
| mock-swarm-1 | schema-and-tool | Pairs a JSON schema agent with a tool call agent, favoring schema output 60/40. |
| mock-swarm-2 | instruction-duo | Two instruction-style agents with equal influence. |

## Functions

| Old Name | New Name | New Description |
|----------|----------|-----------------|
| mock-1 | binary-classifier | Classifies text as yes or no. |
| mock-2 | spam-with-optional-sentiment | Detects whether text is spam, and optionally analyzes its sentiment. |
| mock-3 | five-star-rating | Rates text on a 1-to-5 star scale. |
| mock-4 | item-ranker | Ranks a list of items by overall preference. |
| mock-5 | contextual-ranker | Ranks items by relevance and quality given a search query. |
| mock-6 | email-importance | Determines whether an email is important from its subject line and body. |
| mock-7 | five-criteria-ranker | Ranks items across quality, clarity, relevance, originality, and conciseness. |
| mock-8 | strict-contextual-ranker | Ranks items by relevance and accuracy, with optional strict fact-checking and source reliability criteria. |
| mock-9 | spam-importance-branch | Evaluates text for both spam likelihood and email importance. |
| mock-10 | triple-classifier-branch | Evaluates text using binary classification, five-star rating, and importance scoring. |
| mock-11 | classifier-with-optional-sentiment | Classifies text with an optional importance analysis. |
| mock-12 | dual-ranker-branch | Ranks items using both a simple ranker and a five-criteria ranker. |
| mock-13 | mixed-scalar-vector-branch | Combines text classification scores with item ranking into a single vector output. |
| mock-14 | ranker-with-optional-quality | Ranks items with an optional quality-focused sub-ranker. |
| mock-15 | triple-ranker-branch | Ranks items using three independent ranking strategies. |
| mock-16 | four-way-vector-branch | Ranks items using two classifiers and two rankers for comprehensive evaluation. |
| mock-17 | deep-optional-mixed-branch | Combines classification and ranking with optional deep analysis of email importance and contextual relevance. |
| mock-18 | nested-scalar-super-branch | Two-level nested text evaluation combining the spam-importance and triple-classifier branches. |
| mock-19 | skipable-nested-scalar-branch | Three nested scalar branches for thorough text evaluation, with the middle branch conditional on a thoroughness flag. |
| mock-20 | nested-vector-super-branch | Two-level nested item ranking combining the dual-ranker and triple-ranker branches. |
| mock-21 | contextual-nested-vector-branch | Three nested vector branches for comprehensive context-aware item ranking. |
| mock-22 | mapped-branch-with-votes | Evaluates each item through a spam-importance branch, plus two binary votes on positivity and relevance. |
| mock-23 | mapped-branch-with-classifiers | Evaluates each item through a spam-importance branch, plus tone, sentiment, and intent classifiers. |
| mock-24 | mapped-branch-mixed-tasks | Evaluates each item through a spam-importance branch, runs a standalone binary classifier, and collects two binary votes on quality and clarity. |
| mock-25 | dual-placeholder | Two placeholder tasks that always output fixed 0.5 scores, useful as a baseline. |
| mock-err-1 | error-bad-output-field | Scalar classifier whose output expression references a field that does not exist. |
| mock-err-2 | error-scalar-out-of-range | Scalar classifier whose output expression produces -1.0, outside the valid [0, 1] range. |
| mock-err-3 | error-scalar-returns-vector | Scalar classifier whose output expression returns a vector instead of a scalar. |
| mock-err-4 | error-vector-bad-sum | Vector ranker whose output expression doubles all scores, producing a sum of ~2 instead of ~1. |
| mock-err-5 | error-vector-returns-scalar | Vector ranker whose output expression returns a single scalar instead of a vector. |
| mock-err-6 | error-nested-list-output | Scalar classifier whose output expression returns a nested list instead of a single value. |
| mock-err-7 | error-none-output | Scalar classifier whose output expression returns None. |
| mock-err-8 | error-missing-input-key | Scalar classifier whose task expression references an input key that does not exist. |
| mock-err-9 | error-missing-sub-function | Branch that references a sub-function that does not exist. |
| mock-err-10 | error-wrong-sub-input | Branch that passes incorrect field names to its sub-function. |

## Profiles

| Old Name | New Name | New Description |
|----------|----------|-----------------|
| mock-1 | solo-instruction | A single instruction agent making all decisions. |
| mock-2 | instruction-and-schema | An instruction agent and a JSON schema agent, favoring instruction output 70/30. |
| mock-3 | triple-mode | Three agents — instruction, JSON schema, and tool call — for diverse voting. |
| mock-5 | contextual-duo | An instruction agent and a JSON schema agent weighted 60/40, suited for context-aware ranking. |
| mock-7 | schema-heavy-trio | Two JSON schema agents and one tool call agent with logprobs, for multi-criteria evaluation. |
| mock-8 | logprobs-and-tool | A JSON schema agent with high logprobs paired with a tool call agent for calibrated ranking. |
| mock-9 | schema-logprobs-solo | A single JSON schema agent with logprobs for confident scoring. |
| mock-10 | trio-with-error-agent | Three agents where the tool call agent always errors, testing graceful degradation. |
| mock-11 | tool-and-schema | A tool call agent and a JSON schema agent, weighted 55/45. |
| mock-12 | logprobs-duo | Two JSON schema agents, one with logprobs, for precise vector ranking. |
| mock-13 | schema-solo | A single JSON schema agent for straightforward evaluation. |
| mock-14 | trio-with-error-instruction | Three agents where the instruction agent always errors, testing partial failure. |
| mock-15 | high-logprobs-duo | A JSON schema and tool call agent each with 15 logprobs for high-resolution probabilistic voting. |
| mock-16 | quad-with-error | Four agents where one JSON schema agent always errors, testing resilience in larger ensembles. |
| mock-17 | max-logprobs-duo | A JSON schema and tool call agent each with the maximum 20 logprobs. |
| mock-18 | expanded-nested-scalar | Fully expanded nested profile with inline agents for the first branch and a remote sub-profile for the second. |
| mock-19 | mixed-nested-with-skip | Three nested branches mixing remote and inline sub-profiles for flexible scalar evaluation. |
| mock-20 | nested-vector-inline-remote | Two nested vector tasks — one with inline agents, one delegating to a remote sub-profile. |
| mock-21 | deep-nested-vector | Three deeply nested vector tasks with up to five levels of inline and remote sub-profiles. |
| mock-22 | remote-swarm-mapped-branch | Three tasks delegating to remote swarms that resolve remote agents, for mapped branch execution. |
| mock-23 | remote-swarm-classifiers | Four tasks delegating to remote swarms with remote agents, for mapped branch classification. |
| mock-err-1 | baseline-auto | A single instruction agent with equal weight. |
| mock-err-9 | baseline-tasks | Two tasks with a remote sub-profile reference. |
| mock-err-11 | two-task-tasks | Two instruction agent tasks with equal weights. |
| mock-err-12 | error-weights-length-mismatch | One task but two weights, causing a length validation failure. |
| mock-err-13 | placeholder-and-remote-tasks | A placeholder task paired with a remote sub-profile reference. |
| mock-err-15 | error-dangling-swarm-ref | References a swarm that does not exist. |
| mock-err-16 | error-weight-count-mismatch | One agent but two weights in the swarm definition, causing validation failure. |
| mock-err-17 | dangling-and-valid-tasks | One task referencing a nonexistent sub-profile alongside a valid remote task. |
| mock-err-18 | error-all-agents-fail | A single agent configured to always return an error. |
