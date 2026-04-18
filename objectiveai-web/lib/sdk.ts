import { ObjectiveAI } from "objectiveai";

const API_BASE = "https://api.objective-ai.io";

/** Shared ObjectiveAI client */
let _client: ObjectiveAI | null = null;

export function getClient(): ObjectiveAI {
  if (!_client) {
    _client = new ObjectiveAI({
      apiBase: API_BASE,
    });
  }
  return _client;
}
