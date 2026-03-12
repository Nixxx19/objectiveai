import { ResponseError } from "./responseError";

export { ResponseError };

/**
 * Error thrown when an API request fails.
 *
 * - `body`: The complete ResponseError (contains code and message)
 * - `message` (inherited from Error): JSON-serialized body for stack traces
 */
export class ObjectiveAIFetchError extends Error {
  readonly body: ResponseError;

  /**
   * Construct directly from a ResponseError (e.g., when streaming yields an error).
   */
  constructor(body: ResponseError);
  /**
   * Construct from a status code and optional raw body string.
   *
   * - If rawBody is missing/null/undefined, constructs with null message
   * - If rawBody parses to a ResponseError, uses that (ignores code param)
   * - Otherwise, constructs ResponseError with code and parsed JSON (or raw string) as message
   */
  constructor(code: number, rawBody?: string | null);
  constructor(codeOrBody: number | ResponseError, rawBody?: string | null) {
    let body: ResponseError;

    if (typeof codeOrBody !== "number") {
      // Direct ResponseError
      body = codeOrBody;
    } else if (rawBody === null || rawBody === undefined) {
      // No body, construct with null message
      body = { code: codeOrBody, message: null };
    } else {
      // Try to parse as JSON
      let parsed: unknown;
      try {
        parsed = JSON.parse(rawBody);
      } catch {
        // JSON parsing failed, use raw string as message
        body = { code: codeOrBody, message: rawBody };
        super(JSON.stringify(body));
        this.name = "ObjectiveAIFetchError";
        this.body = body;
        return;
      }

      // Check if parsed is already a ResponseError
      if (isResponseError(parsed)) {
        // Use the parsed ResponseError, ignore the code param
        body = parsed;
      } else {
        // Use parsed JSON as the message
        body = { code: codeOrBody, message: parsed };
      }
    }

    // Error.message is a JSON-serialized ResponseError for complete error info
    super(JSON.stringify(body));
    this.name = "ObjectiveAIFetchError";
    this.body = body;
  }

  /**
   * Convenience getter for the error code.
   */
  get code(): number {
    return this.body.code;
  }

  /**
   * Serialize to ResponseError JSON format.
   */
  toJSON(): ResponseError {
    return this.body;
  }
}

/**
 * Check if an object looks like a ResponseError.
 */
export function isResponseError(obj: unknown): obj is ResponseError {
  return (
    typeof obj === "object" &&
    obj !== null &&
    "code" in obj &&
    typeof (obj as { code: unknown }).code === "number" &&
    "message" in obj
  );
}
