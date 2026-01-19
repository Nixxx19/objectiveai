"use client";

import { useState, useEffect, use } from "react";
import Link from "next/link";

// Mock function data
const FUNCTION_DATA: Record<string, {
  name: string;
  description: string;
  category: string;
  inputs: Array<{ name: string; type: string; description: string }>;
}> = {
  "trip-must-see": {
    name: "Trip Must-See",
    description: "Ranks tourist attractions by local authenticity and visitor satisfaction scores. Great for travel planning and destination curation.",
    category: "Ranking",
    inputs: [
      { name: "destination", type: "text", description: "City or region to analyze" },
      { name: "travelerType", type: "select", description: "Type of traveler" },
      { name: "interests", type: "textarea", description: "Specific interests (comma-separated)" },
    ],
  },
  "email-classifier": {
    name: "Email Classifier",
    description: "Categorizes emails by intent, urgency, and required action type.",
    category: "Scoring",
    inputs: [
      { name: "emailContent", type: "textarea", description: "Email body text" },
      { name: "subject", type: "text", description: "Email subject line" },
    ],
  },
};

const DEFAULT_FUNCTION = {
  name: "Function",
  description: "AI-powered scoring and ranking function",
  category: "General",
  inputs: [
    { name: "input", type: "textarea", description: "Your input data" },
  ],
};

export default function FunctionDetailPage({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = use(params);
  const functionData = FUNCTION_DATA[slug] || DEFAULT_FUNCTION;
  
  const [formData, setFormData] = useState<Record<string, string>>({});
  const [temperature, setTemperature] = useState(0.7);
  const [isRunning, setIsRunning] = useState(false);
  const [isMobile, setIsMobile] = useState(false);
  const [isSaved, setIsSaved] = useState(false);
  const [showPinnedColor, setShowPinnedColor] = useState(false);
  const [results, setResults] = useState<{
    score?: number;
    rankings?: Array<{ name: string; score: number }>;
    breakdown?: Array<{ metric: string; value: number; confidence: number }>;
  } | null>(null);

  // Load saved state from localStorage
  useEffect(() => {
    const savedLibrary = localStorage.getItem('pinned-functions');
    if (savedLibrary) {
      const library = JSON.parse(savedLibrary);
      setIsSaved(library.includes(slug));
    }
  }, [slug]);

  // Toggle save state
  const toggleSave = () => {
    const savedLibrary = localStorage.getItem('pinned-functions');
    const library = savedLibrary ? JSON.parse(savedLibrary) : [];
    
    if (isSaved) {
      const updated = library.filter((s: string) => s !== slug);
      localStorage.setItem('pinned-functions', JSON.stringify(updated));
      setIsSaved(false);
    } else {
      library.push(slug);
      localStorage.setItem('pinned-functions', JSON.stringify(library));
      setIsSaved(true);
      // Flash the pinned color briefly
      setShowPinnedColor(true);
      setTimeout(() => setShowPinnedColor(false), 1000);
    }
  };

  // Track viewport size
  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth <= 768);
    };
    checkMobile();
    window.addEventListener('resize', checkMobile);
    return () => window.removeEventListener('resize', checkMobile);
  }, []);

  const handleRun = () => {
    setIsRunning(true);
    
    // Simulate API call
    setTimeout(() => {
      setResults({
        score: 87.5,
        rankings: [
          { name: "Option A", score: 0.38 },
          { name: "Option B", score: 0.28 },
          { name: "Option C", score: 0.21 },
          { name: "Option D", score: 0.13 },
        ],
        breakdown: [
          { metric: "Quality", value: 0.91, confidence: 0.94 },
          { metric: "Relevance", value: 0.86, confidence: 0.88 },
          { metric: "Clarity", value: 0.82, confidence: 0.91 },
        ],
      });
      setIsRunning(false);
    }, 1500);
  };

  const spinnerStyle = `
    @keyframes spin {
      to { transform: rotate(360deg); }
    }
  `;

  return (
    <div className="page">
      <style dangerouslySetInnerHTML={{ __html: spinnerStyle }} />
      
      <div style={{
        maxWidth: '1400px',
        margin: '0 auto',
        padding: isMobile ? '0 20px' : '0 32px',
      }}>
        {/* Breadcrumb Row with Pin */}
        <div style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'baseline',
          marginBottom: '20px',
          fontSize: '14px',
        }}>
          <nav style={{
            display: 'flex',
            gap: '8px',
            color: 'var(--text-muted)',
            flexWrap: 'wrap',
          }}>
            <Link href="/functions" style={{ color: 'var(--accent)', textDecoration: 'none' }}>
              Functions
            </Link>
            <span>/</span>
            <span>{functionData.name}</span>
          </nav>
          <button
            onClick={toggleSave}
            style={{
              background: 'none',
              border: 'none',
              padding: 0,
              cursor: 'pointer',
              fontSize: 'inherit',
              color: showPinnedColor ? 'var(--accent)' : 'var(--text-muted)',
              opacity: 0.7,
              transition: showPinnedColor ? 'color 0.15s ease-in' : 'color 0.5s ease-out',
            }}
          >
            {isSaved ? 'Pinned Function' : 'Pin Function'}
          </button>
        </div>

        {/* Header */}
        <div style={{ marginBottom: isMobile ? '20px' : '28px' }}>
          <h1 className="heading2" style={{ marginBottom: '4px' }}>
            {functionData.name}
          </h1>
          <p style={{
            fontSize: isMobile ? '15px' : '17px',
            color: 'var(--text-muted)',
            maxWidth: '700px',
            lineHeight: 1.6,
            marginBottom: '8px',
          }}>
            {functionData.description}
          </p>
          <span className="tag" style={{ display: 'inline-block' }}>{functionData.category}</span>
        </div>

        {/* Main Layout */}
        <div style={{
          display: isMobile ? 'flex' : 'grid',
          flexDirection: 'column',
          gridTemplateColumns: '1fr 1fr',
          gap: isMobile ? '24px' : '32px',
          alignItems: 'start',
        }}>
          {/* Left - Input (AI Text Field Format) */}
          <div className="card">
            <h3 style={{
              fontSize: '12px',
              fontWeight: 600,
              marginBottom: '24px',
              textTransform: 'uppercase',
              letterSpacing: '0.05em',
              color: 'var(--text-muted)',
            }}>
              Input
            </h3>
            
            <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
              {functionData.inputs.map(input => (
                <div key={input.name}>
                  <label style={{
                    display: 'block',
                    fontSize: '14px',
                    fontWeight: 600,
                    marginBottom: '8px',
                    color: 'var(--text)',
                  }}>
                    {input.name.charAt(0).toUpperCase() + input.name.slice(1)}
                    <span style={{ 
                      fontWeight: 400, 
                      color: 'var(--text-muted)', 
                      marginLeft: '8px',
                      display: isMobile ? 'block' : 'inline',
                      marginTop: isMobile ? '4px' : '0',
                    }}>
                      {input.description}
                    </span>
                  </label>
                  
                  {input.type === "text" && (
                    <div className="aiTextField">
                      <input
                        type="text"
                        placeholder={`Enter ${input.name.toLowerCase()}...`}
                        value={formData[input.name] || ""}
                        onChange={(e) => setFormData({ ...formData, [input.name]: e.target.value })}
                      />
                      <svg className="arrowIcon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ transform: 'rotate(-90deg)' }}>
                        <path d="M5 12h14M12 5l7 7-7 7" />
                      </svg>
                    </div>
                  )}
                  
                  {input.type === "textarea" && (
                    <div className="aiTextField">
                      <textarea
                        placeholder={`Enter ${input.name.toLowerCase()}...`}
                        value={formData[input.name] || ""}
                        onChange={(e) => setFormData({ ...formData, [input.name]: e.target.value })}
                        rows={4}
                      />
                      <svg className="arrowIcon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ transform: 'rotate(-90deg)' }}>
                        <path d="M5 12h14M12 5l7 7-7 7" />
                      </svg>
                    </div>
                  )}
                  
                  {input.type === "select" && (
                    <select
                      className="select"
                      value={formData[input.name] || ""}
                      onChange={(e) => setFormData({ ...formData, [input.name]: e.target.value })}
                    >
                      <option value="">Select...</option>
                      <option value="solo">Solo Traveler</option>
                      <option value="couple">Couple</option>
                      <option value="family">Family</option>
                      <option value="group">Group</option>
                    </select>
                  )}
                </div>
              ))}
              
              {/* Temperature Slider */}
              <div>
                <label style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  fontSize: '14px',
                  fontWeight: 600,
                  marginBottom: '8px',
                  color: 'var(--text)',
                }}>
                  <span>Temperature</span>
                  <span style={{ color: 'var(--text-muted)', fontWeight: 400 }}>{temperature.toFixed(2)}</span>
                </label>
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.01"
                  value={temperature}
                  onChange={(e) => setTemperature(parseFloat(e.target.value))}
                  style={{
                    width: '100%',
                    accentColor: 'var(--accent)',
                    height: '8px',
                  }}
                />
                <p style={{
                  fontSize: '13px',
                  color: 'var(--text-muted)',
                  marginTop: '6px',
                }}>
                  Controls output randomness. Lower = more deterministic.
                </p>
              </div>
            </div>
            
            <button
              className="pillBtn"
              onClick={handleRun}
              disabled={isRunning}
              style={{
                width: '100%',
                marginTop: '32px',
                opacity: isRunning ? 0.7 : 1,
              }}
            >
              {isRunning ? "Running..." : "Execute"}
            </button>
          </div>

          {/* Right - Results */}
          <div className="card">
            <h3 style={{
              fontSize: '12px',
              fontWeight: 600,
              marginBottom: '24px',
              textTransform: 'uppercase',
              letterSpacing: '0.05em',
              color: 'var(--text-muted)',
            }}>
              Output
            </h3>
            
            {!results && !isRunning && (
              <div style={{
                textAlign: 'center',
                padding: isMobile ? '40px 20px' : '60px 20px',
                color: 'var(--text-muted)',
              }}>
                <p style={{ marginBottom: '8px', fontSize: '24px' }}>—</p>
                <p style={{ fontSize: '14px' }}>Run the function to see results</p>
              </div>
            )}
            
            {isRunning && (
              <div style={{
                textAlign: 'center',
                padding: isMobile ? '40px 20px' : '60px 20px',
                color: 'var(--text-muted)',
              }}>
                <div style={{
                  width: '40px',
                  height: '40px',
                  border: '3px solid var(--border)',
                  borderTopColor: 'var(--accent)',
                  borderRadius: '50%',
                  margin: '0 auto 16px',
                  animation: 'spin 1s linear infinite',
                }} />
                <p style={{ fontSize: '14px' }}>Processing...</p>
              </div>
            )}
            
            {results && !isRunning && (
              <div style={{ display: 'flex', flexDirection: 'column', gap: '32px' }}>
                {/* Score */}
                {results.score && (
                  <div>
                    <p style={{
                      fontSize: '13px',
                      color: 'var(--text-muted)',
                      marginBottom: '8px',
                    }}>
                      Overall Score
                    </p>
                    <p style={{
                      fontSize: isMobile ? '36px' : '48px',
                      fontWeight: 700,
                      color: 'var(--accent)',
                      lineHeight: 1,
                    }}>
                      {results.score.toFixed(1)}%
                    </p>
                  </div>
                )}
                
                {/* Rankings */}
                {results.rankings && (
                  <div>
                    <p style={{
                      fontSize: '13px',
                      color: 'var(--text-muted)',
                      marginBottom: '12px',
                    }}>
                      Rankings
                    </p>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                      {results.rankings.map((item, i) => {
                        // Calculate color based on score (proportional, summing to 100%)
                        // Scale: 40%+ = green, 25% = yellow, 10% = red
                        const getScoreColor = (score: number) => {
                          const pct = score * 100;
                          if (pct >= 35) {
                            // Green (dominant choice)
                            return 'rgb(34, 197, 94)';
                          } else if (pct >= 28) {
                            // Yellow-Green
                            const t = (pct - 28) / 7;
                            return `rgb(${Math.round(132 + (34 - 132) * t)}, ${Math.round(204 + (197 - 204) * t)}, ${Math.round(22 + (94 - 22) * t)})`;
                          } else if (pct >= 20) {
                            // Yellow to Yellow-Green
                            const t = (pct - 20) / 8;
                            return `rgb(${Math.round(234 + (132 - 234) * t)}, ${Math.round(179 + (204 - 179) * t)}, ${Math.round(8 + (22 - 8) * t)})`;
                          } else if (pct >= 12) {
                            // Orange to Yellow
                            const t = (pct - 12) / 8;
                            return `rgb(${Math.round(249 + (234 - 249) * t)}, ${Math.round(115 + (179 - 115) * t)}, ${Math.round(22 + (8 - 22) * t)})`;
                          } else {
                            // Red to Orange
                            const t = pct / 12;
                            return `rgb(${Math.round(239 + (249 - 239) * t)}, ${Math.round(68 + (115 - 68) * t)}, ${Math.round(68 + (22 - 68) * t)})`;
                          }
                        };
                        
                        return (
                          <div key={i} style={{
                            display: 'flex',
                            alignItems: 'center',
                            gap: '12px',
                            padding: '12px 16px',
                            background: 'var(--page-bg)',
                            borderRadius: '12px',
                          }}>
                            {/* Percentage on the left with gradient color */}
                            <span style={{
                              fontSize: '14px',
                              fontWeight: 700,
                              color: getScoreColor(item.score),
                              width: '40px',
                              flexShrink: 0,
                            }}>
                              {(item.score * 100).toFixed(0)}%
                            </span>
                            {/* Option name */}
                            <span style={{ flex: 1, fontSize: '14px' }}>{item.name}</span>
                          </div>
                        );
                      })}
                    </div>
                  </div>
                )}
                
                {/* Breakdown */}
                {results.breakdown && (
                  <div>
                    <p style={{
                      fontSize: '13px',
                      color: 'var(--text-muted)',
                      marginBottom: '12px',
                    }}>
                      Metric Breakdown
                    </p>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                      {results.breakdown.map((item, i) => (
                        <div key={i}>
                          <div style={{
                            display: 'flex',
                            justifyContent: 'space-between',
                            fontSize: '14px',
                            marginBottom: '6px',
                          }}>
                            <span>{item.metric}</span>
                            <span style={{ fontWeight: 600 }}>
                              {(item.value * 100).toFixed(0)}%
                              <span style={{ fontSize: '12px', color: 'var(--text-muted)', marginLeft: '6px' }}>
                                ±{((1 - item.confidence) * 100).toFixed(0)}%
                              </span>
                            </span>
                          </div>
                          <div style={{
                            height: '6px',
                            background: 'var(--border)',
                            borderRadius: '3px',
                            overflow: 'hidden',
                          }}>
                            <div style={{
                              width: `${item.value * 100}%`,
                              height: '100%',
                              background: 'var(--accent)',
                              borderRadius: '3px',
                            }} />
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
