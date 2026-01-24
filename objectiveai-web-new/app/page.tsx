"use client";

import { useState, useEffect, useRef } from "react";
import Link from "next/link";
import HeroText from "@/components/HeroText";

// Featured functions data
const FEATURED_FUNCTIONS = [
  {
    slug: "trip-must-see",
    name: "Trip Must-See",
    description: "Ranks tourist attractions by local authenticity and visitor satisfaction",
    category: "Ranking",
    tags: ["travel", "scoring", "ranking"],
  },
  {
    slug: "email-classifier",
    name: "Email Classifier",
    description: "Categorizes emails by intent, urgency, and required action type",
    category: "Scoring",
    tags: ["text", "classification", "scoring"],
  },
  {
    slug: "code-quality",
    name: "Code Quality",
    description: "Evaluates pull requests across maintainability and security metrics",
    category: "Composite",
    tags: ["code", "evaluation", "scoring"],
  },
];

// Chat message types
type ChatMessage = {
  role: 'user' | 'assistant';
  content: string;
  functionCall?: {
    name: string;
    status: 'thinking' | 'calling' | 'complete';
    result?: Record<string, unknown>;
  };
};

// Initial example conversation
const INITIAL_MESSAGES: ChatMessage[] = [
  {
    role: 'user',
    content: 'Can you analyze this restaurant review? "The pasta was incredible, perfectly al dente with a rich sauce. Service was attentive but not intrusive. Will definitely come back!"',
  },
  {
    role: 'assistant',
    content: 'I\'ll analyze that review for you using the Sentiment Analyzer function.',
    functionCall: {
      name: 'Sentiment Analyzer',
      status: 'complete',
      result: {
        sentiment: 'positive',
        confidence: 0.94,
        themes: ['food quality', 'service', 'intent to return'],
        scores: { food: 0.96, service: 0.89, overall: 0.94 }
      }
    }
  },
  {
    role: 'assistant',
    content: 'The review is strongly positive (94% confidence). Key themes detected: excellent food quality, good service, and clear intent to return. The food received the highest score at 96%.',
  }
];

// Mock responses based on input keywords
const getMockResponse = (input: string): ChatMessage[] => {
  const lower = input.toLowerCase();
  
  if (lower.includes('review') || lower.includes('sentiment') || lower.includes('feedback')) {
    return [
      {
        role: 'assistant',
        content: 'I\'ll analyze that for sentiment and key themes.',
        functionCall: {
          name: 'Sentiment Analyzer',
          status: 'complete',
          result: {
            sentiment: lower.includes('bad') || lower.includes('terrible') ? 'negative' : 'positive',
            confidence: 0.87,
            themes: ['general feedback'],
          }
        }
      },
      {
        role: 'assistant',
        content: `Based on my analysis, the sentiment appears to be ${lower.includes('bad') || lower.includes('terrible') ? 'negative' : 'positive'} with 87% confidence.`,
      }
    ];
  }
  
  if (lower.includes('rank') || lower.includes('compare') || lower.includes('best')) {
    return [
      {
        role: 'assistant',
        content: 'I\'ll rank those options for you.',
        functionCall: {
          name: 'Content Ranker',
          status: 'complete',
          result: {
            ranked: true,
            criteria: ['relevance', 'quality'],
          }
        }
      },
      {
        role: 'assistant',
        content: 'I\'ve ranked the options based on relevance and quality metrics.',
      }
    ];
  }
  
  // Default response
  return [
    {
      role: 'assistant',
      content: 'I can help analyze, rank, or score content for you. Try asking me to analyze a review, rank some options, or evaluate text!',
    }
  ];
};

export default function Home() {
  const [isMobile, setIsMobile] = useState(false);
  const [isTablet, setIsTablet] = useState(false);
  const [messages, setMessages] = useState<ChatMessage[]>(INITIAL_MESSAGES);
  const [inputValue, setInputValue] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);
  const [thinkingText, setThinkingText] = useState('');
  const [messageSent, setMessageSent] = useState(false);
  const messagesContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const checkViewport = () => {
      const width = window.innerWidth;
      setIsMobile(width <= 640);
      setIsTablet(width > 640 && width <= 1024);
    };
    checkViewport();
    window.addEventListener('resize', checkViewport);
    return () => window.removeEventListener('resize', checkViewport);
  }, []);

  // Auto-scroll within chat container only (not page)
  useEffect(() => {
    const container = messagesContainerRef.current;
    if (container) {
      container.scrollTop = container.scrollHeight;
    }
  }, [messages, thinkingText]);

  // Simulate thinking text streaming
  const simulateThinking = async () => {
    const thinkingPhrases = [
      'Analyzing request...',
      'Determining best function...',
      'Routing to handler...',
    ];
    
    for (const phrase of thinkingPhrases) {
      setThinkingText(phrase);
      await new Promise(r => setTimeout(r, 600));
    }
    setThinkingText('');
  };

  const handleSend = async () => {
    if (!inputValue.trim() || isProcessing) return;
    
    // Trigger sent animation
    setMessageSent(true);
    setTimeout(() => setMessageSent(false), 2000);
    
    // Add user message
    setMessages(prev => [...prev, { role: 'user', content: inputValue }]);
    setInputValue('');
    setIsProcessing(true);
    
    // Simulate thinking
    await simulateThinking();
    
    // Get mock response
    const responses = getMockResponse(inputValue);
    
    // Add responses with delays
    for (const response of responses) {
      await new Promise(r => setTimeout(r, 500));
      setMessages(prev => [...prev, response]);
    }
    
    setIsProcessing(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleSend();
    }
  };

  const resetChat = () => {
    setMessages(INITIAL_MESSAGES);
    setInputValue('');
  };

  return (
    <div className="page" style={{
      display: 'flex',
      flexDirection: 'column',
      gap: isMobile ? '80px' : '120px',
      paddingBottom: '60px',
    }}>
      {/* Hero Section */}
      <section style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        minHeight: 'calc(60vh - 100px)',
        paddingTop: isMobile ? '40px' : '60px',
      }}>
        <div style={{ textAlign: 'center', display: 'flex', flexDirection: 'column', alignItems: 'center', width: '100%', padding: '0 32px' }}>
          <div style={{ marginBottom: '32px', width: '100%' }}>
            <HeroText />
          </div>
          <p style={{
            fontSize: isMobile ? '16px' : '18px',
            color: 'var(--text-muted)',
            marginBottom: '32px',
          }}>
            AI Scoring Primitives for Developers
          </p>
          <div style={{
            display: 'flex',
            gap: '16px',
            justifyContent: 'center',
            flexWrap: 'wrap',
          }}>
            <button className="pillBtn">Vibe-Native</button>
            <button className="pillBtn">SDK-First</button>
          </div>
        </div>
      </section>

      {/* Functions Router Demo Section - Chat Interface */}
      <section style={{ padding: '0 32px' }}>
        <div style={{ maxWidth: '800px', margin: '0 auto' }}>
          <div style={{ textAlign: 'center', marginBottom: isMobile ? '32px' : '48px' }}>
            <span className="tag" style={{ marginBottom: '16px', display: 'inline-block' }}>
              Try It
            </span>
            <h2 className="heading2" style={{ marginBottom: '12px' }}>
              Functions Router
            </h2>
            <p style={{ fontSize: '17px', color: 'var(--text-muted)', maxWidth: '600px', margin: '0 auto' }}>
              Chat with AI that intelligently routes to the right function
            </p>
          </div>

          {/* Chat Container */}
          <div className="card" style={{
            padding: 0,
            overflow: 'hidden',
            display: 'flex',
            flexDirection: 'column',
            height: isMobile ? '500px' : '560px',
          }}>
            {/* Chat Header */}
            <div style={{
              padding: '16px 20px',
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}>
              <span style={{ fontSize: '14px', fontWeight: 600 }}>Functions Router</span>
              <button
                onClick={resetChat}
                style={{
                  background: 'none',
                  border: 'none',
                  color: 'var(--text-muted)',
                  cursor: 'pointer',
                  padding: '4px',
                  borderRadius: '6px',
                  transition: 'color 0.2s',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                }}
                onMouseEnter={(e) => e.currentTarget.style.color = 'var(--accent)'}
                onMouseLeave={(e) => e.currentTarget.style.color = 'var(--text-muted)'}
                aria-label="Reset chat"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M1 4v6h6M23 20v-6h-6" />
                  <path d="M20.49 9A9 9 0 0 0 5.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 0 1 3.51 15" />
                </svg>
              </button>
            </div>

            {/* Messages Area */}
            <div
              ref={messagesContainerRef}
              style={{
                flex: 1,
                overflowY: 'auto',
                padding: '20px',
                display: 'flex',
                flexDirection: 'column',
                gap: '16px',
              }}
            >
              {messages.map((msg, index) => (
                <div
                  key={index}
                  style={{
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: msg.role === 'user' ? 'flex-end' : 'flex-start',
                    gap: '8px',
                  }}
                >
                  {/* Message Bubble */}
                  <div style={{
                    maxWidth: '85%',
                    padding: '12px 16px',
                    borderRadius: msg.role === 'user' ? '16px 16px 4px 16px' : '16px 16px 16px 4px',
                    background: msg.role === 'user' ? 'var(--accent)' : 'var(--nav-surface)',
                    color: msg.role === 'user' ? 'var(--color-light)' : 'var(--text)',
                    fontSize: '14px',
                    lineHeight: 1.5,
                  }}>
                    {msg.content}
                  </div>

                  {/* Function Call Card */}
                  {msg.functionCall && (
                    <div style={{
                      maxWidth: '85%',
                      padding: '14px 16px',
                      borderRadius: '12px',
                      background: 'var(--page-bg)',
                      border: '1px solid var(--border)',
                      fontSize: '13px',
                    }}>
                      <div style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '8px',
                        marginBottom: '10px',
                      }}>
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--accent)" strokeWidth="2">
                          <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z" />
                        </svg>
                        <span style={{ fontWeight: 600, color: 'var(--accent)' }}>
                          {msg.functionCall.name}
                        </span>
                        {msg.functionCall.status === 'complete' && (
                          <span style={{
                            fontSize: '11px',
                            padding: '2px 8px',
                            background: 'rgba(107, 92, 255, 0.15)',
                            color: 'var(--accent)',
                            borderRadius: '10px',
                            marginLeft: 'auto',
                          }}>
                            Complete
                          </span>
                        )}
                      </div>
                      {msg.functionCall.result && (
                        <pre style={{
                          margin: 0,
                          padding: '10px 12px',
                          background: 'var(--card-bg)',
                          borderRadius: '8px',
                          fontSize: '12px',
                          fontFamily: 'monospace',
                          color: 'var(--text)',
                          overflow: 'auto',
                          lineHeight: 1.5,
                        }}>
                          {JSON.stringify(msg.functionCall.result, null, 2)}
                        </pre>
                      )}
                    </div>
                  )}
                </div>
              ))}

              {/* Thinking Indicator */}
              {thinkingText && (
                <div style={{
                  display: 'flex',
                  alignItems: 'flex-start',
                }}>
                  <div style={{
                    padding: '12px 16px',
                    borderRadius: '16px 16px 16px 4px',
                    background: 'var(--nav-surface)',
                    fontSize: '14px',
                    color: 'var(--text-muted)',
                    fontStyle: 'italic',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '8px',
                  }}>
                    <span style={{
                      display: 'inline-flex',
                      gap: '3px',
                    }}>
                      <span className="thinkingDot" style={{ animationDelay: '0ms' }}>•</span>
                      <span className="thinkingDot" style={{ animationDelay: '150ms' }}>•</span>
                      <span className="thinkingDot" style={{ animationDelay: '300ms' }}>•</span>
                    </span>
                    {thinkingText}
                  </div>
                </div>
              )}
            </div>

            {/* Input Area */}
            <div style={{
              padding: '16px 20px',
            }}>
              <div className="aiTextField">
                <input
                  type="text"
                  value={inputValue}
                  onChange={(e) => setInputValue(e.target.value)}
                  onKeyDown={handleKeyDown}
                  placeholder="Ask me to analyze, rank, or score something..."
                  disabled={isProcessing}
                />
                <button
                  onClick={handleSend}
                  disabled={!inputValue.trim() || isProcessing}
                  style={{
                    background: 'none',
                    border: 'none',
                    cursor: inputValue.trim() && !isProcessing ? 'pointer' : 'default',
                    padding: 0,
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}
                  aria-label="Send message"
                >
                  <svg 
                    className={`arrowIcon ${messageSent ? 'sent' : ''}`}
                    viewBox="0 0 24 24" 
                    fill="none" 
                    stroke="currentColor" 
                    strokeWidth="2" 
                    style={{ transform: 'rotate(-90deg)' }}
                  >
                    <path d="M5 12h14M12 5l7 7-7 7" />
                  </svg>
                </button>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Thinking dots animation */}
      <style jsx global>{`
        @keyframes thinkingPulse {
          0%, 100% { opacity: 0.3; }
          50% { opacity: 1; }
        }
        .thinkingDot {
          animation: thinkingPulse 1s ease-in-out infinite;
        }
      `}</style>

      {/* Featured Functions Section */}
      <section style={{ padding: '0 32px' }}>
        <div style={{ maxWidth: '1100px', margin: '0 auto' }}>
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

          {/* Function Cards Grid */}
          <div className="gridThree">
            {FEATURED_FUNCTIONS.map(fn => (
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
            ))}
          </div>
        </div>
      </section>
    </div>
  );
}
