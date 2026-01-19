"use client";

import Link from "next/link";
import { useState } from "react";

export default function Footer() {
  const [emailSent, setEmailSent] = useState(false);

  const handleSubmit = () => {
    setEmailSent(true);
    // Reset after 2 seconds
    setTimeout(() => setEmailSent(false), 2000);
  };

  const socialIcons = [
    { 
      name: "GitHub", 
      href: "#", 
      icon: (
        <svg width="18" height="18" viewBox="0 0 98 96" fill="currentColor">
          <path fillRule="evenodd" clipRule="evenodd" d="M48.854 0C21.839 0 0 22 0 49.217c0 21.756 13.993 40.172 33.405 46.69 2.427.49 3.316-1.059 3.316-2.362 0-1.141-.08-5.052-.08-9.127-13.59 2.934-16.42-5.867-16.42-5.867-2.184-5.704-5.42-7.17-5.42-7.17-4.448-3.015.324-3.015.324-3.015 4.934.326 7.523 5.052 7.523 5.052 4.367 7.496 11.404 5.378 14.235 4.074.404-3.178 1.699-5.378 3.074-6.6-10.839-1.141-22.243-5.378-22.243-24.283 0-5.378 1.94-9.778 5.014-13.2-.485-1.222-2.184-6.275.486-13.038 0 0 4.125-1.304 13.426 5.052a46.97 46.97 0 0 1 12.214-1.63c4.125 0 8.33.571 12.213 1.63 9.302-6.356 13.427-5.052 13.427-5.052 2.67 6.763.97 11.816.485 13.038 3.155 3.422 5.015 7.822 5.015 13.2 0 18.905-11.404 23.06-22.324 24.283 1.78 1.548 3.316 4.481 3.316 9.126 0 6.6-.08 11.897-.08 13.526 0 1.304.89 2.853 3.316 2.364 19.412-6.52 33.405-24.935 33.405-46.691C97.707 22 75.788 0 48.854 0z"/>
        </svg>
      )
    },
    { 
      name: "Discord", 
      href: "#", 
      icon: (
        <svg width="18" height="18" viewBox="0 0 127 96" fill="currentColor">
          <path d="M81.15,0c-1.2376,2.1973-2.3489,4.4704-3.3591,6.794-9.5975-1.4396-19.3718-1.4396-28.9945,0-.985-2.3236-2.1216-4.5967-3.3591-6.794-9.0166,1.5407-17.8059,4.2431-26.1405,8.0568C2.779,32.5304-1.6914,56.3725.5312,79.8863c9.6732,7.1476,20.5083,12.603,32.0505,16.0884,2.6014-3.4854,4.8998-7.1981,6.8698-11.0623-3.738-1.3891-7.3497-3.1318-10.8098-5.1523.9092-.6567,1.7932-1.3386,2.6519-1.9953,20.281,9.547,43.7696,9.547,64.0758,0,.8587.7072,1.7427,1.3891,2.6519,1.9953-3.4601,2.0457-7.0718,3.7632-10.835,5.1776,1.97,3.8642,4.2683,7.5769,6.8698,11.0623,11.5419-3.4854,22.3769-8.9156,32.0509-16.0631,2.626-27.2771-4.496-50.9172-18.817-71.8548C98.9811,4.2684,90.1918,1.5659,81.1752.0505l-.0252-.0505ZM42.2802,65.4144c-6.2383,0-11.4159-5.6575-11.4159-12.6535s4.9755-12.6788,11.3907-12.6788,11.5169,5.708,11.4159,12.6788c-.101,6.9708-5.026,12.6535-11.3907,12.6535ZM84.3576,65.4144c-6.2637,0-11.3907-5.6575-11.3907-12.6535s4.9755-12.6788,11.3907-12.6788,11.4917,5.708,11.3906,12.6788c-.101,6.9708-5.026,12.6535-11.3906,12.6535Z"/>
        </svg>
      )
    },
    { 
      name: "X", 
      href: "#", 
      icon: (
        <svg width="16" height="16" viewBox="0 0 1200 1227" fill="currentColor">
          <path d="M714.163 519.284L1160.89 0H1055.03L667.137 450.887L357.328 0H0L468.492 681.821L0 1226.37H105.866L515.491 750.218L842.672 1226.37H1200L714.137 519.284H714.163ZM569.165 687.828L521.697 619.934L144.011 79.6944H306.615L611.412 515.685L658.88 583.579L1055.08 1150.3H892.476L569.165 687.854V687.828Z"/>
        </svg>
      )
    },
    { 
      name: "LinkedIn", 
      href: "#", 
      icon: (
        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
          <path d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433c-1.144 0-2.063-.926-2.063-2.065 0-1.138.92-2.063 2.063-2.063 1.14 0 2.064.925 2.064 2.063 0 1.139-.925 2.065-2.064 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z"/>
        </svg>
      )
    },
    { 
      name: "YouTube", 
      href: "#", 
      icon: (
        <svg width="18" height="18" viewBox="0 0 397 278" fill="currentColor">
          <path d="M388.13,43.45c-4.64-17.09-17.67-30.41-35.05-35.05C322.38,0,198.12,0,198.12,0c0,0-123.97,0-154.67,8.4C26.36,13.03,13.03,26.36,8.11,43.45,0,74.15,0,138.74,0,138.74c0,0,0,64.59,8.11,95.59,4.92,16.8,18.25,30.41,35.34,35.05,30.7,8.4,154.67,8.4,154.67,8.4,0,0,124.26,0,154.96-8.4,17.38-4.63,30.41-18.25,35.05-35.05,8.4-30.99,8.4-95.59,8.4-95.59,0,0,0-64.59-8.4-95.29ZM158.73,198.41v-119.05l102.83,59.38-102.83,59.67Z"/>
        </svg>
      )
    },
  ];

  return (
    <footer className="footer" style={{ padding: '32px 0 6px' }}>
      {/* Footer Module Container */}
      <div style={{
        maxWidth: '1100px',
        margin: '0 auto',
        padding: '0 32px',
      }}>
        <div style={{
          padding: '12px 24px 4px',
        }}>
          {/* ROW 1: Menu + Socials on same row */}
          <div style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'flex-start',
            marginBottom: '28px',
            gap: '48px',
          }} className="footerMenuSocialRow">
            {/* Pages Menu (3 groups) - LEFT */}
            <nav style={{
              display: 'flex',
              gap: '72px',
              paddingLeft: '8px',
            }} className="footerPageMenu">
              {/* GROUP 1: FUNCTIONS */}
              <div style={{ textAlign: 'left' }}>
                <Link
                  href="/functions"
                  style={{
                    display: 'block',
                    fontSize: '10px',
                    fontWeight: 600,
                    textTransform: 'uppercase',
                    letterSpacing: '0.06em',
                    color: 'var(--text-muted)',
                    textDecoration: 'none',
                    marginBottom: '6px',
                    opacity: 0.8,
                    transition: 'color 0.2s',
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.color = 'var(--accent)';
                    e.currentTarget.style.opacity = '1';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.color = 'var(--text-muted)';
                    e.currentTarget.style.opacity = '0.8';
                  }}
                >
                  Functions
                </Link>
                <div style={{
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '3px',
                }}>
                <Link
                  href="/ensembles"
                  style={{
                    fontSize: '12px',
                    fontWeight: 400,
                    color: 'var(--text-muted)',
                    textDecoration: 'none',
                    transition: 'color 0.2s',
                    opacity: 0.7,
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.color = 'var(--accent)';
                    e.currentTarget.style.opacity = '1';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.color = 'var(--text-muted)';
                    e.currentTarget.style.opacity = '0.7';
                  }}
                >
                  Ensembles
                </Link>
                <Link
                  href="/studio"
                  style={{
                    fontSize: '12px',
                    fontWeight: 400,
                    color: 'var(--text-muted)',
                    textDecoration: 'none',
                    transition: 'color 0.2s',
                    opacity: 0.7,
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.color = 'var(--accent)';
                    e.currentTarget.style.opacity = '1';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.color = 'var(--text-muted)';
                    e.currentTarget.style.opacity = '0.7';
                  }}
                >
                  Studio
                </Link>
              </div>
              </div>

              {/* GROUP 2: PEOPLE */}
              <div style={{ textAlign: 'left' }}>
                <Link
                  href="/people"
                  style={{
                    display: 'block',
                    fontSize: '10px',
                    fontWeight: 600,
                    textTransform: 'uppercase',
                    letterSpacing: '0.06em',
                    color: 'var(--text-muted)',
                    textDecoration: 'none',
                    marginBottom: '6px',
                    opacity: 0.8,
                    transition: 'color 0.2s',
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.color = 'var(--accent)';
                    e.currentTarget.style.opacity = '1';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.color = 'var(--text-muted)';
                    e.currentTarget.style.opacity = '0.8';
                  }}
                >
                  People
                </Link>
                <div style={{
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '3px',
                }}>
                <Link
                  href="/team"
                  style={{
                    fontSize: '12px',
                    fontWeight: 400,
                    color: 'var(--text-muted)',
                    textDecoration: 'none',
                    transition: 'color 0.2s',
                    opacity: 0.7,
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.color = 'var(--accent)';
                    e.currentTarget.style.opacity = '1';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.color = 'var(--text-muted)';
                    e.currentTarget.style.opacity = '0.7';
                  }}
                >
                  Team
                </Link>
                <Link
                  href="/contact"
                  style={{
                    fontSize: '12px',
                    fontWeight: 400,
                    color: 'var(--text-muted)',
                    textDecoration: 'none',
                    transition: 'color 0.2s',
                    opacity: 0.7,
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.color = 'var(--accent)';
                    e.currentTarget.style.opacity = '1';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.color = 'var(--text-muted)';
                    e.currentTarget.style.opacity = '0.7';
                  }}
                >
                  Contact
                </Link>
              </div>
              </div>

              {/* GROUP 3: RESOURCES */}
              <div style={{ textAlign: 'left' }}>
                <Link
                  href="/resources"
                  style={{
                    display: 'block',
                    fontSize: '10px',
                    fontWeight: 600,
                    textTransform: 'uppercase',
                    letterSpacing: '0.06em',
                    color: 'var(--text-muted)',
                    textDecoration: 'none',
                    marginBottom: '6px',
                    opacity: 0.8,
                    transition: 'color 0.2s',
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.color = 'var(--accent)';
                    e.currentTarget.style.opacity = '1';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.color = 'var(--text-muted)';
                    e.currentTarget.style.opacity = '0.8';
                  }}
                >
                  Resources
                </Link>
                <div style={{
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '3px',
                }}>
                <Link
                  href="/docs"
                  style={{
                    fontSize: '12px',
                    fontWeight: 400,
                    color: 'var(--text-muted)',
                    textDecoration: 'none',
                    transition: 'color 0.2s',
                    opacity: 0.7,
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.color = 'var(--accent)';
                    e.currentTarget.style.opacity = '1';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.color = 'var(--text-muted)';
                    e.currentTarget.style.opacity = '0.7';
                  }}
                >
                  Docs
                </Link>
                <Link
                  href="/legal"
                  style={{
                    fontSize: '12px',
                    fontWeight: 400,
                    color: 'var(--text-muted)',
                    textDecoration: 'none',
                    transition: 'color 0.2s',
                    opacity: 0.7,
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.color = 'var(--accent)';
                    e.currentTarget.style.opacity = '1';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.color = 'var(--text-muted)';
                    e.currentTarget.style.opacity = '0.7';
                  }}
                >
                  Legal
                </Link>
              </div>
              </div>
            </nav>

            {/* Social Icons - RIGHT */}
            <div style={{
              display: 'flex',
              gap: '20px',
              alignItems: 'flex-start',
              flexShrink: 0,
              paddingRight: '8px',
            }} className="footerSocialStrip">
              {socialIcons.map((social) => (
                <a
                  key={social.name}
                  href={social.href}
                  style={{
                    width: '40px',
                    height: '40px',
                    borderRadius: '50%',
                    border: '1px solid var(--border)',
                    background: 'var(--card-bg)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    color: 'var(--text)',
                    textDecoration: 'none',
                    fontSize: '14px',
                    fontWeight: 600,
                    transition: 'all 0.2s',
                    flexShrink: 0,
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.borderColor = 'var(--accent)';
                    e.currentTarget.style.background = 'rgba(107, 92, 255, 0.05)';
                    e.currentTarget.style.color = 'var(--accent)';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.borderColor = 'var(--border)';
                    e.currentTarget.style.background = 'var(--card-bg)';
                    e.currentTarget.style.color = 'var(--text)';
                  }}
                  aria-label={social.name}
                >
                  {social.icon}
                </a>
              ))}
            </div>
          </div>

          {/* ROW 2: Support Bar */}
          <div style={{
            paddingTop: '8px',
            marginBottom: '28px',
          }} className="footerSupportBar">
            <label style={{
              display: 'block',
              fontSize: '10px',
              fontWeight: 600,
              marginBottom: '8px',
              textTransform: 'uppercase',
              letterSpacing: '0.06em',
              color: 'var(--text-muted)',
              opacity: 0.7,
            }}>
              Support & Inquiries
            </label>
            <div className="humanTextField" style={{ 
              maxWidth: '100%',
            }}>
              <input
                type="email"
                placeholder="you@email.com"
                style={{ 
                  width: '100%',
                }}
              />
              <button onClick={handleSubmit} aria-label="Send">
                <svg 
                  className={`arrowIcon ${emailSent ? 'sent' : ''}`}
                  viewBox="0 0 24 24" 
                  fill="none" 
                  stroke="currentColor" 
                  strokeWidth="2"
                >
                  <path d="M5 12h14M12 5l7 7-7 7" />
                </svg>
              </button>
            </div>
          </div>

          {/* ROW 3: Identity */}
          <div style={{
            display: 'flex',
            alignItems: 'center',
            gap: '10px',
          }} className="footerIdentity">
            {/* Logo Mark - aligned with "O" in Objective */}
            <svg 
              width="20" 
              height="13" 
              viewBox="0 0 180 120.85" 
              fill="currentColor"
              style={{ opacity: 0.5 }}
            >
              <path d="M3.01,67.4H0v-13.94h3.01c6.02,0,10.94-4.92,10.94-10.94v-17.64C13.94,11.21,25.15,0,38.82,0h7.66v13.94h-7.66c-6.02,0-10.94,4.92-10.94,10.94v17.64c0,6.97-3.01,13.4-7.66,17.91,4.65,4.51,7.66,10.94,7.66,17.91v17.64c0,6.01,4.92,10.94,10.94,10.94h7.66v13.94h-7.66c-13.67,0-24.88-11.21-24.88-24.88v-17.64c0-6.02-4.92-10.94-10.94-10.94Z"/>
              <path d="M159.77,60.42c-4.65-4.51-7.66-10.94-7.66-17.91v-17.64c0-6.02-4.92-10.94-10.94-10.94h-7.66V0h7.66c13.67,0,24.88,11.21,24.88,24.88v17.64c0,6.02,4.92,10.94,10.94,10.94h3.01v13.94h-3.01c-6.02,0-10.94,4.92-10.94,10.94v17.64c0,13.67-11.21,24.88-24.88,24.88h-7.66v-13.94h7.66c6.02,0,10.94-4.92,10.94-10.94v-17.64c0-6.97,3.01-13.4,7.66-17.91Z"/>
              <path d="M51.38,53.6c.42-.49,1.29-1.22,2.59-2.17,1.3-.95,2.96-1.9,4.97-2.86,2.01-.95,4.36-1.78,7.04-2.49,2.68-.7,5.61-1.06,8.78-1.06s6.37.37,9.37,1.11c3,.74,5.66,1.96,7.99,3.65,2.33,1.69,4.18,3.91,5.56,6.67s2.06,6.14,2.06,10.16v31.43h-14.82l-1.27-6.03c-1.62,2.26-3.67,4.01-6.14,5.24-2.47,1.23-5.54,1.85-9.21,1.85-2.82,0-5.31-.41-7.46-1.22-2.15-.81-3.95-1.92-5.4-3.33-1.45-1.41-2.54-3.07-3.28-4.97-.74-1.91-1.11-3.95-1.11-6.14s.48-4.41,1.43-6.46c.95-2.04,2.47-3.83,4.55-5.34,2.08-1.52,4.78-2.73,8.1-3.65,3.32-.92,7.34-1.38,12.06-1.38h6.14v-.42c0-2.12-.76-3.77-2.28-4.98-1.52-1.2-4-1.8-7.46-1.8-1.55,0-3.05.21-4.5.64-1.45.42-2.75.92-3.92,1.48-1.16.57-2.17,1.15-3.02,1.75-.85.6-1.45,1.08-1.8,1.43l-8.99-11.11ZM83.34,75.82h-5.5c-3.53,0-5.96.65-7.3,1.96-1.34,1.31-2.01,2.77-2.01,4.39,0,1.2.46,2.33,1.38,3.39.92,1.06,2.4,1.59,4.45,1.59.85,0,1.8-.21,2.86-.63,1.06-.42,2.03-1.06,2.91-1.91.88-.85,1.64-1.92,2.28-3.23.63-1.3.95-2.8.95-4.5v-1.06Z"/>
              <path d="M106.94,32.22c0-1.55.28-3,.85-4.34.56-1.34,1.32-2.5,2.28-3.49.95-.99,2.08-1.76,3.39-2.33,1.3-.56,2.7-.85,4.18-.85s2.87.28,4.18.85c1.3.57,2.47,1.34,3.49,2.33,1.02.99,1.82,2.15,2.38,3.49.56,1.34.85,2.79.85,4.34s-.28,2.91-.85,4.29c-.57,1.38-1.36,2.56-2.38,3.54-1.02.99-2.19,1.78-3.49,2.38-1.31.6-2.7.9-4.18.9s-2.88-.3-4.18-.9c-1.31-.6-2.43-1.39-3.39-2.38-.95-.99-1.71-2.17-2.28-3.54-.57-1.38-.85-2.8-.85-4.29ZM108.95,46.19h17.46v51.86h-17.46v-51.86Z"/>
            </svg>
            <div style={{
              fontSize: '11px',
              color: 'var(--text-muted)',
              fontWeight: 500,
              letterSpacing: '0.01em',
              opacity: 0.6,
            }}>
              Objective Artificial Intelligence, 2025
            </div>
          </div>
        </div>
      </div>

      {/* Responsive styles */}
      <style jsx global>{`
        @media (max-width: 1024px) {
          .footerPageMenu {
            gap: 48px !important;
          }
        }
        
        @media (max-width: 640px) {
          .footerMenuSocialRow {
            flex-direction: column !important;
            gap: 24px !important;
          }
          
          .footerSocialStrip {
            flex-wrap: wrap;
            gap: 10px !important;
          }
          
          .footerPageMenu {
            flex-direction: column !important;
            gap: 16px !important;
          }
        }
      `}</style>
    </footer>
  );
}
