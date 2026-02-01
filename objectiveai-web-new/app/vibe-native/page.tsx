"use client";

import { useState, useEffect } from "react";
import Link from "next/link";

const CLAUDE_HYPERPROMPT = `Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.

## Section One
Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.

## Section Two
Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.

## Section Three
Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium.`;

export default function VibeNativePage() {
  const [isMobile, setIsMobile] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    const checkViewport = () => setIsMobile(window.innerWidth <= 640);
    checkViewport();
    window.addEventListener('resize', checkViewport);
    return () => window.removeEventListener('resize', checkViewport);
  }, []);

  const copyPrompt = () => {
    navigator.clipboard.writeText(CLAUDE_HYPERPROMPT);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="page">
      <div className="container">
        {/* Header */}
        <div style={{ marginBottom: '48px' }}>
          <span className="tag" style={{ marginBottom: '12px', display: 'inline-block' }}>
            No-Code Path
          </span>
          <h1 className="heading1" style={{ marginBottom: '12px' }}>
            Vibe-Native
          </h1>
          <p style={{
            color: 'var(--text-muted)',
            fontSize: '16px',
            maxWidth: '600px',
            lineHeight: 1.6,
          }}>
            Use ObjectiveAI functions without writing code. Try functions directly in the browser or use the Claude hyperprompt below.
          </p>
        </div>

        {/* Try Functions CTA */}
        <div className="card" style={{
          padding: isMobile ? '24px 20px' : '32px',
          marginBottom: '24px',
          textAlign: 'center',
        }}>
          <h2 style={{
            fontSize: isMobile ? '18px' : '20px',
            fontWeight: 600,
            marginBottom: '12px',
          }}>
            Try Functions in Browser
          </h2>
          <p style={{
            color: 'var(--text-muted)',
            fontSize: '14px',
            marginBottom: '20px',
            maxWidth: '400px',
            margin: '0 auto 20px',
          }}>
            Test scoring and ranking functions directly. No sign-up required.
          </p>
          <Link
            href="/functions"
            className="pillBtn"
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: '8px',
              padding: '14px 28px',
              fontSize: '15px',
              fontWeight: 600,
              background: 'var(--accent)',
              color: 'var(--color-light)',
              textDecoration: 'none',
            }}
          >
            Browse Functions
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M5 12h14M12 5l7 7-7 7" />
            </svg>
          </Link>
        </div>

        {/* Claude Hyperprompt */}
        <div style={{ marginBottom: '24px' }}>
          <h2 style={{
            fontSize: isMobile ? '18px' : '20px',
            fontWeight: 600,
            marginBottom: '12px',
          }}>
            Claude Hyperprompt
          </h2>
          <p style={{
            color: 'var(--text-muted)',
            fontSize: '14px',
            marginBottom: '16px',
          }}>
            Copy this system prompt to use with Claude for natural language interaction with ObjectiveAI.
          </p>
          <div style={{
            position: 'relative',
            background: 'var(--nav-surface)',
            border: '1px solid var(--border)',
            borderRadius: '12px',
            overflow: 'hidden',
          }}>
            <div style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              padding: '8px 16px',
              borderBottom: '1px solid var(--border)',
              background: 'var(--card-bg)',
            }}>
              <span style={{
                fontSize: '12px',
                color: 'var(--text-muted)',
                fontFamily: 'var(--font-mono)',
              }}>
                system prompt
              </span>
              <button
                onClick={copyPrompt}
                style={{
                  background: 'none',
                  border: 'none',
                  padding: '4px 8px',
                  cursor: 'pointer',
                  color: copied ? 'var(--accent)' : 'var(--text-muted)',
                  fontSize: '12px',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '4px',
                }}
              >
                {copied ? 'Copied' : 'Copy'}
              </button>
            </div>
            <pre style={{
              margin: 0,
              padding: isMobile ? '16px' : '20px',
              overflow: 'auto',
              fontSize: isMobile ? '12px' : '13px',
              lineHeight: 1.6,
              fontFamily: 'var(--font-mono)',
              maxHeight: '400px',
              whiteSpace: 'pre-wrap',
              wordBreak: 'break-word',
            }}>
              <code>{CLAUDE_HYPERPROMPT}</code>
            </pre>
          </div>
        </div>

        {/* How it works */}
        <div className="card" style={{
          padding: isMobile ? '20px' : '24px',
          background: 'var(--nav-surface)',
        }}>
          <h3 style={{
            fontSize: '15px',
            fontWeight: 600,
            marginBottom: '12px',
          }}>
            How to Use
          </h3>
          <ol style={{
            fontSize: '14px',
            color: 'var(--text-muted)',
            lineHeight: 1.7,
            paddingLeft: '20px',
            margin: 0,
          }}>
            <li>Copy the system prompt above</li>
            <li>Paste it into Claude as a system prompt or project instruction</li>
            <li>Ask Claude to help you score or rank things</li>
            <li>Claude will guide you to the right function and explain the output</li>
          </ol>
        </div>
      </div>
    </div>
  );
}
