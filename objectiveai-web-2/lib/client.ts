const API_BASE = "https://api.objective-ai.io";

/** Fetch JSON from the ObjectiveAI API */
export async function apiFetch<T>(path: string): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`);
  if (!res.ok) throw new Error(`API ${path}: ${res.status}`);
  return res.json();
}
