import Link from "next/link";

export default function Demo() {
  return (
    <main style={{ padding: 48, fontFamily: "var(--font-mono)", fontSize: 12, color: "var(--info-dim)" }}>
      <p>Demo page retired. View live trees on{" "}
        <Link href="/functions" style={{ color: "var(--copper-mid)" }}>function detail pages</Link>.
      </p>
    </main>
  );
}
