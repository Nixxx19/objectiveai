"use client";

import { useState, useEffect, useCallback } from "react";
import dynamic from "next/dynamic";
import Link from "next/link";
import { Functions } from "objectiveai";
import { createPublicClient } from "../lib/client";
import { deriveCategory, deriveDisplayName } from "../lib/objectiveai";
import { useResponsive } from "../hooks/useResponsive";

// Lazy-load the 3D hero scene (heavy, uses WebGL)
const HeroScene = dynamic(() => import("@/components/hero3d/HeroScene"), {
  ssr: false,
  loading: () => null, // Fallback handled inside HeroScene itself
});

// =============================================================================
// FEATURED FUNCTIONS CONFIGURATION
// =============================================================================
const FEATURED_COUNT = 3;
const INSTALL_COMMAND = "npm i -g @objectiveai/cli";

interface FeaturedFunction {
  slug: string;
  name: string;
  description: string;
  category: string;
  tags: string[];
}

function CopyIcon({ size = 16 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
    </svg>
  );
}

function CheckIcon({ size = 16 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="20 6 9 17 4 12" />
    </svg>
  );
}

export default function Home() {
  const { isMobile } = useResponsive();
  const [copied, setCopied] = useState(false);
  const [slots, setSlots] = useState<(FeaturedFunction | null)[]>(
    Array.from({ length: FEATURED_COUNT }, () => null)
  );
  const [isListLoading, setIsListLoading] = useState(true);

  // Copy install command
  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(INSTALL_COMMAND);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Fallback for older browsers
      const textarea = document.createElement("textarea");
      textarea.value = INSTALL_COMMAND;
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      document.body.removeChild(textarea);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  }, []);

  // Fetch functions from API — progressive loading
  useEffect(() => {
    let cancelled = false;

    async function fetchFunctions() {
      try {
        setIsListLoading(true);

        const client = createPublicClient();
        const result = await Functions.list(client);

        // Deduplicate by owner/repository
        const uniqueFunctions = new Map<string, { owner: string; repository: string; commit: string }>();
        for (const fn of result.data) {
          const key = `${fn.owner}/${fn.repository}`;
          if (!uniqueFunctions.has(key)) {
            uniqueFunctions.set(key, fn);
          }
        }

        const entries = Array.from(uniqueFunctions.values()).slice(0, FEATURED_COUNT);
        if (cancelled) return;

        setSlots(Array.from({ length: entries.length }, () => null));
        setIsListLoading(false);

        entries.forEach((fn, index) => {
          const controller = new AbortController();
          const timeout = setTimeout(() => controller.abort(), 5000);

          Functions.retrieve(client, "github", fn.owner, fn.repository, fn.commit, { signal: controller.signal })
            .then((details) => {
              clearTimeout(timeout);
              if (cancelled) return;

              const name = deriveDisplayName(fn.repository);
              const tags = fn.repository.split("-").filter((t: string) => t.length > 2);
              if (details.type === "vector.function") tags.push("ranking");
              else tags.push("scoring");

              const item: FeaturedFunction = {
                slug: `${fn.owner}/${fn.repository}`,
                name,
                description: details.description || `${name} function`,
                category: deriveCategory(details),
                tags,
              };

              setSlots(prev => {
                const next = [...prev];
                next[index] = item;
                return next;
              });
            })
            .catch(() => {
              clearTimeout(timeout);
            });
        });
      } catch {
        if (!cancelled) {
          setIsListLoading(false);
        }
      }
    }

    fetchFunctions();
    return () => { cancelled = true; };
  }, []);

  return (
    <div className="page" style={{
      display: 'flex',
      flexDirection: 'column',
      gap: isMobile ? '80px' : '120px',
      paddingBottom: '60px',
    }}>
      {/* Hero Section — Thinker + Atmospheric Flow into Hyperprompt */}
      <section className="hero">
        {/* Atmospheric glow */}
        <div className="hero-atmosphere" />

        {/* 3D Thinker Council Scene */}
        <div className="hero-scene-wrap">
          <HeroScene />
        </div>

        {/* Copy + Terminal Command */}
        <div className="hero-content">
          <h1 className="hero-tagline">
            <span>Score everything.</span>
            <span>Rank everything.</span>
            <span>Simulate anyone.</span>
          </h1>

          <p className="hero-description">
            Ensembles of LLMs vote on your inputs to produce objective, weighted scores. Build scoring pipelines from your terminal.
          </p>

          {/* Terminal install command */}
          <div className="hero-terminal">
            <span className="terminal-prompt">$</span>
            <code>{INSTALL_COMMAND}</code>
            <button
              className={`hero-terminal-copy ${copied ? "copied" : ""}`}
              onClick={handleCopy}
              aria-label="Copy install command"
            >
              {copied ? <CheckIcon size={14} /> : <CopyIcon size={14} />}
            </button>
          </div>

          <div className="hero-ctas">
            <a
              href="https://github.com/ObjectiveAI/objectiveai"
              target="_blank"
              rel="noopener noreferrer"
              className="pillBtn"
            >
              GitHub
            </a>
            <Link href="/functions" className="pillBtnGhost" style={{ textDecoration: 'none' }}>
              Browse Functions
            </Link>
          </div>
        </div>
      </section>

      {/* Featured Functions Section */}
      <section>
        <div className="container">
          <div style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'flex-end',
            marginBottom: isMobile ? '24px' : '32px',
            flexWrap: 'wrap',
            gap: '16px',
          }}>
            <div>
              <span className="tag" style={{ marginBottom: '12px', display: 'inline-block' }}>
                Explore
              </span>
              <h2 className="heading2">Featured Functions</h2>
            </div>
            <Link
              href="/functions"
              style={{
                fontSize: '15px',
                fontWeight: 600,
                color: 'var(--accent)',
                textDecoration: 'none',
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
              }}
            >
              View all <span>→</span>
            </Link>
          </div>

          {/* Function Cards Grid — slots fill progressively */}
          <div className="gridThree">
            {slots.map((fn, i) => fn ? (
              <Link
                key={fn.slug}
                href={`/functions/${fn.slug}`}
                style={{ textDecoration: 'none', color: 'inherit' }}
              >
                <div className="card" style={{
                  cursor: 'pointer',
                  height: '100%',
                  display: 'flex',
                  flexDirection: 'column',
                  position: 'relative',
                  padding: '16px',
                }}>
                  <span className="tag" style={{
                    alignSelf: 'flex-start',
                    marginBottom: '8px',
                    fontSize: '11px',
                    padding: '4px 10px'
                  }}>
                    {fn.category}
                  </span>
                  <h3 style={{ fontSize: '16px', fontWeight: 600, marginBottom: '6px' }}>
                    {fn.name}
                  </h3>
                  <p style={{
                    fontSize: '13px',
                    lineHeight: 1.5,
                    color: 'var(--text-muted)',
                    flex: 1,
                    marginBottom: '12px',
                    display: '-webkit-box',
                    WebkitLineClamp: 2,
                    WebkitBoxOrient: 'vertical',
                    overflow: 'hidden',
                  }}>
                    {fn.description}
                  </p>
                  <div style={{
                    display: 'flex',
                    flexWrap: 'wrap',
                    gap: '4px',
                    marginBottom: '10px',
                  }}>
                    {fn.tags.slice(0, 2).map(tag => (
                      <span key={tag} style={{
                        fontSize: '11px',
                        padding: '3px 8px',
                        background: 'var(--border)',
                        borderRadius: '10px',
                        color: 'var(--text-muted)',
                      }}>
                        {tag}
                      </span>
                    ))}
                    {fn.tags.length > 2 && (
                      <span style={{
                        fontSize: '11px',
                        padding: '3px 8px',
                        color: 'var(--text-muted)',
                      }}>
                        +{fn.tags.length - 2}
                      </span>
                    )}
                  </div>
                  <div style={{
                    fontSize: '13px',
                    fontWeight: 600,
                    color: 'var(--accent)',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '4px',
                  }}>
                    Open <span>→</span>
                  </div>
                </div>
              </Link>
            ) : (
              <div key={i} className="card" style={{
                padding: '16px',
                height: '180px',
                display: 'flex',
                flexDirection: 'column',
                gap: '8px',
              }}>
                <div style={{ width: '60px', height: '20px', background: 'var(--border)', borderRadius: '10px', animation: 'pulse 1.5s ease-in-out infinite' }} />
                <div style={{ width: '80%', height: '18px', background: 'var(--border)', borderRadius: '4px', animation: 'pulse 1.5s ease-in-out infinite' }} />
                <div style={{ width: '100%', height: '32px', background: 'var(--border)', borderRadius: '4px', animation: 'pulse 1.5s ease-in-out infinite' }} />
              </div>
            ))}
            {!isListLoading && slots.length === 0 && (
              <div style={{
                gridColumn: '1 / -1',
                textAlign: 'center',
                padding: '48px 24px',
                color: 'var(--text-muted)',
              }}>
                <p>No functions available yet.</p>
                <Link
                  href="/functions"
                  style={{
                    color: 'var(--accent)',
                    textDecoration: 'none',
                    fontWeight: 500,
                  }}
                >
                  Browse all functions →
                </Link>
              </div>
            )}
          </div>
        </div>
      </section>

    </div>
  );
}
