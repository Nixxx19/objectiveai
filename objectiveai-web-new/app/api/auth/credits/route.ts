import { NextResponse } from "next/server";
import { ObjectiveAI, Auth } from "objectiveai";

// Server-side client with API key from environment
function getServerClient(): ObjectiveAI {
  return new ObjectiveAI({
    apiKey: process.env.OBJECTIVEAI_API_KEY,
  });
}

// GET /api/auth/credits - Get credit balance
export async function GET() {
  try {
    const client = getServerClient();
    const result = await Auth.Credits.retrieve(client);
    return NextResponse.json(result);
  } catch (error) {
    const message = error instanceof Error ? error.message : "Unknown error";
    const status = message.includes("401") ? 401 : 500;
    return NextResponse.json({ error: message }, { status });
  }
}
