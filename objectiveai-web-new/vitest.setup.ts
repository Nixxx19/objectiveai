// Vitest setup file
// Add global test setup here

// Import jest-dom matchers for component tests
// This adds matchers like toBeInTheDocument, toHaveClass, etc.
// Only needed when @testing-library/jest-dom is installed
try {
  // @ts-expect-error - jest-dom may not be installed
  require("@testing-library/jest-dom/vitest");
} catch {
  // jest-dom not available, skip
}
