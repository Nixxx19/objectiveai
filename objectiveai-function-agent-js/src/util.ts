// Get the current working directory with forward slashes (Linux/Mac style)
export function getSlashCwd(): string {
  return process.cwd().replace(/\\/g, "/");
}

// Get the current working directory with backslashes (Windows style)
export function getBackslashCwd(): string {
  return process.cwd().replace(/\//g, "\\");
}
