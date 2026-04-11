import OpenAI from "openai";

const API_BASE = "https://api.objective-ai.io";

/** Shared OpenAI-compatible client for the ObjectiveAI API */
let _client: OpenAI | null = null;

export function getClient(): OpenAI {
  if (!_client) {
    _client = new OpenAI({
      baseURL: API_BASE,
      apiKey: "", // public endpoints don't require auth
      dangerouslyAllowBrowser: true,
    });
  }
  return _client;
}
