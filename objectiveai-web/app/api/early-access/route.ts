import { NextResponse } from "next/server";

export async function POST(request: Request) {
  const { email } = await request.json();

  if (!email || !email.includes("@")) {
    return NextResponse.json({ error: "Invalid email" }, { status: 400 });
  }

  const apiKey = process.env.BUTTONDOWN_API_KEY;
  if (!apiKey) {
    // No Buttondown key configured — accept silently for dev/preview
    return NextResponse.json({ success: true });
  }

  try {
    const res = await fetch("https://api.buttondown.com/v1/subscribers", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Token ${apiKey}`,
      },
      body: JSON.stringify({
        email_address: email,
        tags: ["early-access"],
      }),
    });

    if (res.ok || res.status === 201) {
      return NextResponse.json({ success: true });
    }

    if (res.status === 409) {
      return NextResponse.json({ already_subscribed: true });
    }

    return NextResponse.json(
      { error: "Something went wrong" },
      { status: 500 },
    );
  } catch {
    return NextResponse.json(
      { error: "Something went wrong" },
      { status: 500 },
    );
  }
}
