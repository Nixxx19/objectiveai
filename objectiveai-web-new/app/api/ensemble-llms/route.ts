import { NextResponse } from "next/server";
import { ObjectiveAI, EnsembleLlm } from "objectiveai";
import { normalizeError, getErrorStatusCode } from "@/lib/error-handling";

function getServerClient(): ObjectiveAI {
  return new ObjectiveAI({
    apiKey: process.env.OBJECTIVEAI_API_KEY,
  });
}

export async function GET() {
  try {
    const client = getServerClient();
    const result = await EnsembleLlm.list(client);
    return NextResponse.json(result);
  } catch (error) {
    const message = normalizeError(error);
    const statusCode = getErrorStatusCode(error);
    return NextResponse.json({ error: message }, { status: statusCode });
  }
}
