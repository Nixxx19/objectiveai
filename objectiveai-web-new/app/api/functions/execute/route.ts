import { NextRequest, NextResponse } from "next/server";
import { ObjectiveAI, Functions } from "objectiveai";

// Server-side client with API key from environment
function getServerClient(): ObjectiveAI {
  return new ObjectiveAI({
    apiKey: process.env.OBJECTIVEAI_API_KEY,
  });
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { functionRef, profileRef, input, options } = body;

    if (!functionRef || !profileRef) {
      return NextResponse.json(
        { error: "Missing functionRef or profileRef" },
        { status: 400 }
      );
    }

    const client = getServerClient();

    const result = await Functions.Executions.create(
      client,
      functionRef,
      profileRef,
      { input, ...options }
    );

    return NextResponse.json(result);
  } catch (error) {
    const message = error instanceof Error ? error.message : "Unknown error";
    const status = message.includes("401") ? 401 : 500;
    return NextResponse.json({ error: message }, { status });
  }
}
