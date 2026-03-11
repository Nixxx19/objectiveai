"use client";

import { Suspense, useState, useEffect, useRef, useCallback } from "react";
import { Canvas } from "@react-three/fiber";
import ThinkerCouncil from "./ThinkerCouncil";

// ---------------------------------------------------------------------------
// Scroll-phase boundaries
// ---------------------------------------------------------------------------
// Phase 0 (0.0–0.2): Idle — orb neutral, score ~0.3
// Phase 1 (0.2–0.4): Awareness — subtle head tilts
// Phase 2 (0.4–0.6): Deliberation — staggered tilts increase
// Phase 3 (0.6–0.8): Voting — figures nod
// Phase 4 (0.8–1.0): Resolved — score climbs to 1.0, green

// ---------------------------------------------------------------------------
// Static fallback shown while R3F loads (or if WebGL unavailable)
// ---------------------------------------------------------------------------
function StaticFallback() {
  return (
    // eslint-disable-next-line @next/next/no-img-element
    <img
      src="/hero/thinker.webp"
      srcSet="/hero/thinker.webp 1x, /hero/thinker@2x.webp 2x"
      alt="ObjectiveAI"
      className="hero-thinker"
      draggable={false}
      style={{
        width: "100%",
        height: "100%",
        objectFit: "contain",
        objectPosition: "center",
      }}
    />
  );
}

// ---------------------------------------------------------------------------
// HeroScene
// ---------------------------------------------------------------------------
export default function HeroScene() {
  const containerRef = useRef<HTMLDivElement>(null);
  const scrollProgressRef = useRef(0);
  const isVisibleRef = useRef(true);
  const rafIdRef = useRef<number>(0);

  const [isVisible, setIsVisible] = useState(true);
  const [prefersReducedMotion, setPrefersReducedMotion] = useState(false);
  const [isClient, setIsClient] = useState(false);

  // SSR guard — Canvas must only render on the client
  useEffect(() => {
    setIsClient(true);
  }, []);

  // Detect prefers-reduced-motion
  useEffect(() => {
    const mql = window.matchMedia("(prefers-reduced-motion: reduce)");
    setPrefersReducedMotion(mql.matches);

    const handler = (e: MediaQueryListEvent) =>
      setPrefersReducedMotion(e.matches);
    mql.addEventListener("change", handler);
    return () => mql.removeEventListener("change", handler);
  }, []);

  // IntersectionObserver — toggle rendering when offscreen
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        const visible = entry.isIntersecting;
        isVisibleRef.current = visible;
        setIsVisible(visible);
      },
      { threshold: 0 }
    );

    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  // Scroll-driven progress calculation
  const updateScrollProgress = useCallback(() => {
    const el = containerRef.current;
    if (!el || !isVisibleRef.current) return;

    const rect = el.getBoundingClientRect();
    const windowHeight = window.innerHeight;

    // progress = 0 when hero is fully in view at the bottom of the viewport
    // progress = 1 when hero has scrolled completely above the viewport
    // Clamp to [0, 1]
    const rawProgress = 1 - (rect.bottom / (windowHeight + rect.height));
    scrollProgressRef.current = Math.max(0, Math.min(1, rawProgress));

    rafIdRef.current = requestAnimationFrame(updateScrollProgress);
  }, []);

  useEffect(() => {
    if (prefersReducedMotion) {
      // Reduced motion: lock at fully-resolved state
      scrollProgressRef.current = 1;
      return;
    }

    rafIdRef.current = requestAnimationFrame(updateScrollProgress);
    return () => {
      if (rafIdRef.current) cancelAnimationFrame(rafIdRef.current);
    };
  }, [prefersReducedMotion, updateScrollProgress]);

  // Before hydration, show static fallback
  if (!isClient) {
    return (
      <div
        ref={containerRef}
        style={{
          width: "100%",
          height: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <StaticFallback />
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
      style={{ width: "100%", height: "100%", position: "relative" }}
    >
      <Suspense
        fallback={
          <div
            style={{
              width: "100%",
              height: "100%",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            <StaticFallback />
          </div>
        }
      >
        <Canvas
          camera={{ position: [0, 0.5, 8], fov: 35 }}
          dpr={[1, 2]}
          frameloop={isVisible ? "always" : "never"}
          gl={{
            antialias: true,
            alpha: true,
            powerPreference: "high-performance",
          }}
          style={{ background: "transparent" }}
        >
          <ThinkerCouncil
            scrollProgressRef={scrollProgressRef}
            reducedMotion={prefersReducedMotion}
          />
        </Canvas>
      </Suspense>
    </div>
  );
}
