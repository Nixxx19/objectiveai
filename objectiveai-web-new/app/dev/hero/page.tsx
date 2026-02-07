"use client";

import dynamic from "next/dynamic";

// Dynamic import — Canvas uses WebGL, SSR must be off
const HeroScene = dynamic(
  () => import("@/components/hero3d/HeroScene"),
  { ssr: false }
);

/**
 * Dev-only preview page for the 3D hero figure.
 * Visit /dev/hero to inspect the thinker figure with orbit controls.
 */
export default function HeroDevPage() {
  return (
    <div style={{
      width: "100vw",
      height: "100vh",
      position: "fixed",
      inset: 0,
      zIndex: 9999,
      background: "#1B1B1B",
    }}>
      <HeroScene style={{ width: "100%", height: "100%" }} />

      {/* Controls hint */}
      <div style={{
        position: "absolute",
        bottom: 24,
        left: "50%",
        transform: "translateX(-50%)",
        color: "rgba(255,255,255,0.3)",
        fontSize: 13,
        fontFamily: "var(--font-geist-mono), monospace",
        pointerEvents: "none",
        userSelect: "none",
      }}>
        drag to orbit · scroll to zoom
      </div>
    </div>
  );
}
