import { NextRequest, NextResponse } from "next/server";
import { ObjectiveAI, Auth } from "objectiveai";

// Server-side client with API key from environment
function getServerClient(): ObjectiveAI {
  return new ObjectiveAI({
    apiKey: process.env.OBJECTIVEAI_API_KEY,
  });
}

// GET /api/auth/keys - List all API keys
export async function GET() {
  try {
    const client = getServerClient();
    const result = await Auth.ApiKey.list(client);
    return NextResponse.json(result);
  } catch (error) {
    const message = error instanceof Error ? error.message : "Unknown error";
    const status = message.includes("401") ? 401 : 500;
    return NextResponse.json({ error: message }, { status });
  }
}

// POST /api/auth/keys - Create a new API key
export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { name, expires, description } = body;

    if (!name) {
      return NextResponse.json(
        { error: "Missing required field: name" },
        { status: 400 }
      );
    }

    const client = getServerClient();
    const result = await Auth.ApiKey.create(
      client,
      name,
      expires ? new Date(expires) : null,
      description || null
    );
    return NextResponse.json(result);
  } catch (error) {
    const message = error instanceof Error ? error.message : "Unknown error";
    const status = message.includes("401") ? 401 : 500;
    return NextResponse.json({ error: message }, { status });
  }
}

// DELETE /api/auth/keys - Disable an API key
export async function DELETE(request: NextRequest) {
  try {
    const body = await request.json();
    const { api_key } = body;

    if (!api_key) {
      return NextResponse.json(
        { error: "Missing required field: api_key" },
        { status: 400 }
      );
    }

    const client = getServerClient();
    const result = await Auth.ApiKey.disable(client, api_key);
    return NextResponse.json(result);
  } catch (error) {
    const message = error instanceof Error ? error.message : "Unknown error";
    const status = message.includes("401") ? 401 : 500;
    return NextResponse.json({ error: message }, { status });
  }
}
