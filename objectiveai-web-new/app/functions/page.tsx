"use client";

import { useState, useEffect, useRef } from "react";
import Link from "next/link";

// Mock functions data - easily extendable
const MOCK_FUNCTIONS = [
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
  {
    slug: "resume-matcher",
    name: "Resume Matcher",
    description: "Scores candidate resumes against job requirements with skill gap analysis",
    category: "Ranking",
    tags: ["hr", "matching", "ranking"],
  },
  {
    slug: "content-ranker",
    name: "Content Ranker",
    description: "Ranks content by engagement potential, SEO value, and audience fit",
    category: "Ranking",
    tags: ["content", "seo", "ranking"],
  },
  {
    slug: "sentiment-analyzer",
    name: "Sentiment Analyzer",
    description: "Analyzes text sentiment with confidence intervals and theme extraction",
    category: "Scoring",
    tags: ["text", "sentiment", "scoring"],
  },
  {
    slug: "data-transformer",
    name: "Data Transformer",
    description: "Transforms unstructured data into structured formats with validation",
    category: "Transformation",
    tags: ["data", "transformation", "validation"],
  },
  {
    slug: "model-benchmark",
    name: "Model Benchmark",
    description: "Benchmarks LLM outputs across quality, cost, and latency metrics",
    category: "Composite",
    tags: ["llm", "benchmark", "evaluation"],
  },
];

const CATEGORIES = ["All", "Pinned", "Scoring", "Ranking", "Transformation", "Composite"];

// Sticky bar height constant
const STICKY_BAR_HEIGHT = 72;


const INITIAL_VISIBLE_COUNT = 6;
const LOAD_MORE_COUNT = 6;

export default function FunctionsPage() {
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategory, setSelectedCategory] = useState("All");
  const [sortBy, setSortBy] = useState("name");
  const [filtersOpen, setFiltersOpen] = useState(false);
  const [pinnedFunctions, setPinnedFunctions] = useState<string[]>([]);
  const [recentFunctions, setRecentFunctions] = useState<string[]>([]);
  const [navOffset, setNavOffset] = useState(96);
  const [isMobile, setIsMobile] = useState(false);
  const [isTablet, setIsTablet] = useState(false);
  const [visibleCount, setVisibleCount] = useState(INITIAL_VISIBLE_COUNT);
  const searchRef = useRef<HTMLDivElement>(null);

  // Track viewport size
  useEffect(() => {
    const checkViewport = () => {
      setIsMobile(window.innerWidth <= 768);
      setIsTablet(window.innerWidth <= 1024);
    };
    checkViewport();
    window.addEventListener('resize', checkViewport);
    return () => window.removeEventListener('resize', checkViewport);
  }, []);

  // Load pinned and recent from localStorage
  useEffect(() => {
    const savedPinned = localStorage.getItem('pinned-functions');
    const savedRecent = localStorage.getItem('recent-functions');
    if (savedPinned) {
      setPinnedFunctions(JSON.parse(savedPinned));
    }
    if (savedRecent) setRecentFunctions(JSON.parse(savedRecent));
  }, []);

  // Dynamic sticky offset calculation based on nav height
  useEffect(() => {
    const updateOffset = () => {
      const navHeightStr = getComputedStyle(document.documentElement).getPropertyValue('--nav-height-actual');
      const navHeight = navHeightStr ? parseInt(navHeightStr) : (isMobile ? 84 : 96);
      setNavOffset(navHeight);
    };
    
    updateOffset();
    window.addEventListener('resize', updateOffset);
    const timer = setTimeout(updateOffset, 100);
    return () => {
      window.removeEventListener('resize', updateOffset);
      clearTimeout(timer);
    };
  }, [isMobile]);

  // Reset visible count when filters change
  useEffect(() => {
    setVisibleCount(INITIAL_VISIBLE_COUNT);
  }, [searchQuery, selectedCategory, sortBy]);

  // Filter and sort functions
  const filteredFunctions = MOCK_FUNCTIONS
    .filter(fn => {
      const matchesSearch = searchQuery === "" ||
        fn.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        fn.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
        fn.tags.some(tag => tag.toLowerCase().includes(searchQuery.toLowerCase()));
      const matchesCategory = selectedCategory === "All" || 
        (selectedCategory === "Pinned" ? pinnedFunctions.includes(fn.slug) : fn.category === selectedCategory);
      return matchesSearch && matchesCategory;
    })
    .sort((a, b) => {
      if (sortBy === "name") return a.name.localeCompare(b.name);
      if (sortBy === "category") return a.category.localeCompare(b.category);
      return 0;
    });

  // Visible functions (paginated)
  const visibleFunctions = filteredFunctions.slice(0, visibleCount);
  const hasMore = visibleCount < filteredFunctions.length;

  const recentFunctionsList = MOCK_FUNCTIONS.filter(fn => recentFunctions.includes(fn.slug));

  // Safe gap from CSS variable
  const safeGap = 24;
  
  // Calculate sticky positions
  const searchBarTop = navOffset;
  const sidebarTop = navOffset + STICKY_BAR_HEIGHT + safeGap;

  return (
    <div className="page">
      <div style={{
        maxWidth: '1400px',
        margin: '0 auto',
        padding: isMobile ? '0 20px' : '0 32px',
      }}>
        {/* Header */}
        <div style={{ marginBottom: isMobile ? '24px' : '32px' }}>
          <h1 className="heading2" style={{ marginBottom: '8px' }}>Functions</h1>
          <p style={{ fontSize: isMobile ? '15px' : '17px', color: 'var(--text-muted)' }}>
            Browse AI functions for scoring, ranking, and evaluation
          </p>
        </div>

        {/* Sticky Search Bar Row with Filter Button */}
        <div
          ref={searchRef}
          style={{
            position: 'sticky',
            top: `${searchBarTop}px`,
            zIndex: 100,
            display: 'flex',
            alignItems: 'center',
            gap: '12px',
            marginBottom: safeGap,
            background: 'var(--page-bg)',
            padding: '8px 0',
          }}
        >
          {/* Filter/Sort Button - Left of Search Bar */}
          {!isMobile && (
            <button
              className="iconBtn"
              onClick={() => setFiltersOpen(!filtersOpen)}
              aria-label={filtersOpen ? "Close filters" : "Open filters"}
              style={{ flexShrink: 0 }}
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M3 6h18M7 12h10M11 18h2" />
              </svg>
            </button>
          )}

          {/* Search Bar - Full Pill Shape */}
          <div className="searchBarPill" style={{ flex: 1 }}>
            <input
              type="text"
              placeholder="Search functions..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
            <svg className="searchIcon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="11" cy="11" r="8" />
              <path d="M21 21l-4.35-4.35" />
            </svg>
          </div>
        </div>

        {/* Layout - responsive, full width when filters collapsed */}
        <div style={{
          display: isMobile ? 'block' : (filtersOpen ? 'grid' : 'block'),
          gridTemplateColumns: filtersOpen ? (isTablet ? '220px 1fr' : '280px 1fr') : undefined,
          gap: isTablet ? '24px' : '32px',
          alignItems: 'start',
        }}>
          {/* Left Sidebar - Filters - Collapsible */}
          {!isMobile && filtersOpen && (
            <aside
              className="stickySidebar"
              style={{
                position: 'sticky',
                top: `${sidebarTop}px`,
                padding: '20px',
              }}
            >
              <h3 style={{
                fontSize: '12px',
                fontWeight: 600,
                marginBottom: '12px',
                textTransform: 'uppercase',
                letterSpacing: '0.05em',
                color: 'var(--text-muted)',
              }}>
                Category
              </h3>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', marginBottom: '20px' }}>
                {CATEGORIES.map(cat => (
                  <button
                    key={cat}
                    onClick={() => setSelectedCategory(cat)}
                    className={`filterChip ${selectedCategory === cat ? 'active' : ''}`}
                    style={{ 
                      textAlign: 'left', 
                      padding: '8px 14px',
                      opacity: cat === 'Pinned' && pinnedFunctions.length === 0 ? 0.5 : 1,
                    }}
                    disabled={cat === 'Pinned' && pinnedFunctions.length === 0}
                  >
                    {cat === 'Pinned' ? `Pinned (${pinnedFunctions.length})` : cat}
                  </button>
                ))}
              </div>

              <h3 style={{
                fontSize: '12px',
                fontWeight: 600,
                marginBottom: '12px',
                textTransform: 'uppercase',
                letterSpacing: '0.05em',
                color: 'var(--text-muted)',
              }}>
                Sort By
              </h3>
              <select
                value={sortBy}
                onChange={(e) => setSortBy(e.target.value)}
                className="select"
              >
                <option value="name">Name</option>
                <option value="category">Category</option>
              </select>
            </aside>
          )}

          {/* Function Cards Grid - Compact tiles */}
          <div>
            <div style={{
              display: 'grid',
              gridTemplateColumns: isMobile 
                ? '1fr' 
                : isTablet 
                  ? 'repeat(2, 1fr)' 
                  : filtersOpen 
                    ? 'repeat(2, 1fr)'
                    : 'repeat(3, 1fr)',
              gap: isMobile ? '12px' : '16px',
            }}>
              {visibleFunctions.map(fn => (
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
                    <span className="tag" style={{ alignSelf: 'flex-start', marginBottom: '8px', fontSize: '11px', padding: '4px 10px' }}>
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

            {/* Load More */}
            {hasMore && (
              <button
                onClick={() => setVisibleCount(prev => prev + LOAD_MORE_COUNT)}
                style={{
                  display: 'block',
                  width: '100%',
                  padding: '16px',
                  marginTop: '24px',
                  background: 'none',
                  border: 'none',
                  fontSize: '14px',
                  fontWeight: 600,
                  color: 'var(--accent)',
                  cursor: 'pointer',
                  textAlign: 'center',
                  transition: 'opacity 0.2s',
                }}
                onMouseEnter={(e) => e.currentTarget.style.opacity = '0.7'}
                onMouseLeave={(e) => e.currentTarget.style.opacity = '1'}
              >
                Load more ({filteredFunctions.length - visibleCount} remaining)
              </button>
            )}

            {filteredFunctions.length === 0 && (
              <div style={{
                textAlign: 'center',
                padding: '60px 20px',
                color: 'var(--text-muted)',
              }}>
                No functions found matching your criteria
              </div>
            )}
          </div>
        </div>

        {/* Mobile Filter Overlay */}
        {filtersOpen && isMobile && (
          <>
            <div
              style={{
                position: 'fixed',
                top: 0,
                left: 0,
                right: 0,
                bottom: 0,
                background: 'rgba(27, 27, 27, 0.7)',
                zIndex: 200,
              }}
              onClick={() => setFiltersOpen(false)}
            />
            <div style={{
              position: 'fixed',
              bottom: 0,
              left: 0,
              right: 0,
              background: 'var(--card-bg)',
              zIndex: 201,
              padding: '24px',
              borderTopLeftRadius: '20px',
              borderTopRightRadius: '20px',
              boxShadow: '0 -4px 20px var(--shadow)',
              maxHeight: '70vh',
              overflowY: 'auto',
            }}>
              <div style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                marginBottom: '20px',
              }}>
                <h3 style={{ fontSize: '18px', fontWeight: 600 }}>Filters</h3>
                <button
                  onClick={() => setFiltersOpen(false)}
                  style={{
                    background: 'none',
                    border: 'none',
                    fontSize: '24px',
                    cursor: 'pointer',
                    color: 'var(--text)',
                  }}
                >
                  ✕
                </button>
              </div>
              
              <h4 style={{
                fontSize: '12px',
                fontWeight: 600,
                marginBottom: '12px',
                textTransform: 'uppercase',
                letterSpacing: '0.05em',
                color: 'var(--text-muted)',
              }}>
                Category
              </h4>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px', marginBottom: '24px' }}>
                {CATEGORIES.map(cat => (
                  <button
                    key={cat}
                    onClick={() => {
                      setSelectedCategory(cat);
                      setFiltersOpen(false);
                    }}
                    className={`filterChip ${selectedCategory === cat ? 'active' : ''}`}
                    style={{ 
                      opacity: cat === 'Pinned' && pinnedFunctions.length === 0 ? 0.5 : 1,
                    }}
                    disabled={cat === 'Pinned' && pinnedFunctions.length === 0}
                  >
                    {cat === 'Pinned' ? `Pinned (${pinnedFunctions.length})` : cat}
                  </button>
                ))}
              </div>

              <h4 style={{
                fontSize: '12px',
                fontWeight: 600,
                marginBottom: '12px',
                textTransform: 'uppercase',
                letterSpacing: '0.05em',
                color: 'var(--text-muted)',
              }}>
                Sort By
              </h4>
              <select
                value={sortBy}
                onChange={(e) => {
                  setSortBy(e.target.value);
                  setFiltersOpen(false);
                }}
                className="select"
              >
                <option value="name">Name</option>
                <option value="category">Category</option>
              </select>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
