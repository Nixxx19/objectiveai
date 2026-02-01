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

    // Handle streaming
    if (options?.stream) {
      const stream = await Functions.Executions.create(
        client,
        functionRef,
        profileRef,
        { input, ...options, stream: true }
      );

      // Create a readable stream that forwards SSE chunks
      const encoder = new TextEncoder();
      const readable = new ReadableStream({
        async start(controller) {
          try {
            for await (const chunk of stream) {
              const data = `data: ${JSON.stringify(chunk)}\n\n`;
              controller.enqueue(encoder.encode(data));
            }
            controller.enqueue(encoder.encode("data: [DONE]\n\n"));
            controller.close();
          } catch (err) {
            const message = err instanceof Error ? err.message : "Stream error";
            controller.enqueue(encoder.encode(`data: ${JSON.stringify({ error: message })}\n\n`));
            controller.close();
          }
        },
      });

      return new Response(readable, {
        headers: {
          "Content-Type": "text/event-stream",
          "Cache-Control": "no-cache",
          "Connection": "keep-alive",
        },
      });
    }

    // Non-streaming
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
