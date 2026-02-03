"use client";

import { useState, useEffect } from "react";

/**
 * Breakpoints matching globals.css
 * @see objectiveai-web-new/app/globals.css
 */
const MOBILE_BREAKPOINT = 640;
const TABLET_BREAKPOINT = 1024;

interface ResponsiveState {
  /** Viewport is mobile width (<= 640px) */
  isMobile: boolean;
  /** Viewport is tablet width or smaller (<= 1024px) */
  isTablet: boolean;
  /** Viewport is desktop width (> 1024px) */
  isDesktop: boolean;
  /** Current window width in pixels */
  windowWidth: number;
}

/**
 * Custom hook for detecting responsive breakpoints.
 * Consolidates mobile and tablet detection that was previously
 * duplicated across browse pages.
 *
 * Breakpoints match globals.css:
 * - Mobile: <= 640px
 * - Tablet: <= 1024px
 * - Desktop: > 1024px
 *
 * @example
 * const { isMobile, isTablet, isDesktop } = useResponsive();
 * // Use for responsive layouts
 * const columns = isDesktop ? 3 : isTablet ? 2 : 1;
 */
export function useResponsive(): ResponsiveState {
  const [state, setState] = useState<ResponsiveState>({
    isMobile: false,
    isTablet: false,
    isDesktop: true,
    windowWidth: typeof window !== "undefined" ? window.innerWidth : 1200,
  });

  useEffect(() => {
    // Check if window is available (SSR safety)
    if (typeof window === "undefined") return;

    const updateState = () => {
      const width = window.innerWidth;
      setState({
        isMobile: width <= MOBILE_BREAKPOINT,
        isTablet: width <= TABLET_BREAKPOINT,
        isDesktop: width > TABLET_BREAKPOINT,
        windowWidth: width,
      });
    };

    // Initial check
    updateState();

    // Listen for resize events
    window.addEventListener("resize", updateState);

    // Cleanup
    return () => window.removeEventListener("resize", updateState);
  }, []);

  return state;
}

export default useResponsive;
