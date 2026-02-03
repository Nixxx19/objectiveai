import { execSync } from "child_process";
import { existsSync, readFileSync } from "fs";

export interface IssueComment {
  body: string;
  created_at: string;
  user: { login: string } | null;
}

export interface Issue {
  number: number;
  title: string;
  body: string | null;
  state: "open" | "closed";
  labels: { name: string }[];
  created_at: string;
  updated_at: string;
  closed_at: string | null;
  user: { login: string } | null;
  comments?: IssueComment[];
}

// Execute gh command and return output
function gh(args: string): string {
  return execSync(`gh ${args}`, { encoding: "utf-8", stdio: "pipe" }).trim();
}

// Get the upstream remote URL (origin)
function getUpstream(): string | null {
  try {
    return execSync("git remote get-url origin", {
      encoding: "utf-8",
      stdio: "pipe",
    }).trim();
  } catch {
    return null;
  }
}

// Fetch comments for an issue
export function fetchIssueComments(issueNumber: number): IssueComment[] {
  const result = gh(
    `issue view ${issueNumber} --json comments`,
  );
  const raw = JSON.parse(result);

  return (raw.comments || []).map((comment: Record<string, unknown>) => ({
    body: comment.body,
    created_at: comment.createdAt,
    user: comment.author ? { login: (comment.author as { login: string }).login } : null,
  }));
}

// Check if there are any open issues
export function hasOpenIssues(): boolean {
  const upstream = getUpstream();
  if (!upstream) {
    return false;
  }

  try {
    const result = gh("issue list --state open --json number --limit 1");
    const issues = JSON.parse(result);
    return issues.length > 0;
  } catch {
    return false;
  }
}

// Fetch all open issues (with comments)
export function fetchOpenIssues(): Issue[] {
  const upstream = getUpstream();
  if (!upstream) {
    throw new Error("No upstream remote found. Cannot fetch issues.");
  }

  const result = gh(
    "issue list --state open --json number,title,body,state,labels,createdAt,updatedAt,closedAt,author",
  );
  const raw = JSON.parse(result);

  return raw.map((issue: Record<string, unknown>) => ({
    number: issue.number,
    title: issue.title,
    body: issue.body,
    state: issue.state,
    labels: issue.labels,
    created_at: issue.createdAt,
    updated_at: issue.updatedAt,
    closed_at: issue.closedAt,
    user: issue.author ? { login: (issue.author as { login: string }).login } : null,
    comments: fetchIssueComments(issue.number as number),
  }));
}

// Fetch all closed issues (with comments)
export function fetchClosedIssues(): Issue[] {
  const upstream = getUpstream();
  if (!upstream) {
    throw new Error("No upstream remote found. Cannot fetch issues.");
  }

  const result = gh(
    "issue list --state closed --json number,title,body,state,labels,createdAt,updatedAt,closedAt,author",
  );
  const raw = JSON.parse(result);

  return raw.map((issue: Record<string, unknown>) => ({
    number: issue.number,
    title: issue.title,
    body: issue.body,
    state: issue.state,
    labels: issue.labels,
    created_at: issue.createdAt,
    updated_at: issue.updatedAt,
    closed_at: issue.closedAt,
    user: issue.author ? { login: (issue.author as { login: string }).login } : null,
    comments: fetchIssueComments(issue.number as number),
  }));
}

// Comment on an issue
export function commentOnIssue(issueNumber: number, comment: string): void {
  const upstream = getUpstream();
  if (!upstream) {
    throw new Error("No upstream remote found. Cannot comment on issue.");
  }

  // Use stdin to pass the comment body to avoid shell escaping issues
  execSync(`gh issue comment ${issueNumber} --body-file -`, {
    input: comment,
    encoding: "utf-8",
    stdio: ["pipe", "pipe", "pipe"],
  });
}

// Mark an issue as ready to close (doesn't actually close it)
// Returns the issue number for later closing
export function markIssueResolved(issueNumber: number): number {
  const upstream = getUpstream();
  if (!upstream) {
    throw new Error("No upstream remote found. Cannot mark issue resolved.");
  }
  // Just validate the issue exists
  gh(`issue view ${issueNumber} --json number`);
  return issueNumber;
}

// Actually close an issue (call this after successful build/test)
export function closeIssue(issueNumber: number): void {
  const upstream = getUpstream();
  if (!upstream) {
    throw new Error("No upstream remote found. Cannot close issue.");
  }

  gh(`issue close ${issueNumber}`);
}

// Read JSON file, return null if it doesn't exist
function readJsonFile(path: string): unknown {
  if (!existsSync(path)) {
    return null;
  }
  const content = readFileSync(path, "utf-8");
  return JSON.parse(content);
}

export interface CreateRepositoryOptions {
  name?: string;
  description?: string;
  public?: boolean;
}

// Create a GitHub repository
export function createRepository(options: CreateRepositoryOptions = {}): string {
  // Get name from options or fallback to github/name.json
  const name = options.name ?? (readJsonFile("github/name.json") as string | null);
  if (!name) {
    throw new Error("Repository name is required. Provide it as option or in github/name.json");
  }

  // Get description from options or fallback to github/description.json
  const description = options.description ?? (readJsonFile("github/description.json") as string | null);

  // Build the command
  let cmd = `repo create ${name}`;
  if (description) {
    cmd += ` --description "${description.replace(/"/g, '\\"')}"`;
  }
  if (options.public !== false) {
    cmd += " --public";
  } else {
    cmd += " --private";
  }
  cmd += " --source=. --push";

  gh(cmd);

  // Return the new repo URL
  return getUpstream() ?? `https://github.com/${name}`;
}

export interface CommitAndPushOptions {
  message: string;
  name?: string;
  description?: string;
  dryRun?: boolean; // If true, commit but don't push
}

// Commit all changes and optionally push
export function commitAndPush(options: CommitAndPushOptions): void {
  const { message, dryRun = false } = options;

  // Stage all changes
  execSync("git add -A", { stdio: "pipe" });

  // Check if there are changes to commit
  try {
    execSync("git diff --cached --quiet", { stdio: "pipe" });
    // No changes to commit
    console.log("No changes to commit.");
    return;
  } catch {
    // There are changes, commit them
    execSync(`git commit -m "${message.replace(/"/g, '\\"')}"`, { stdio: "inherit" });
  }

  // If dry run, don't push
  if (dryRun) {
    console.log("Dry run: commit created but not pushed.");
    return;
  }

  // Check if there's an upstream remote
  const upstream = getUpstream();

  // If no upstream, create repository
  if (!upstream) {
    createRepository({
      name: options.name,
      description: options.description,
    });
  } else {
    // Push to existing remote
    execSync("git push", { stdio: "inherit" });
  }
}

// Commit only (no push) - for use in mainLoop
export function commitOnly(message: string): void {
  commitAndPush({ message, dryRun: true });
}

// Push existing commits
export function push(): void {
  const upstream = getUpstream();
  if (!upstream) {
    throw new Error("No upstream remote found. Cannot push.");
  }
  execSync("git push", { stdio: "inherit" });
}

// Push existing commits, creating upstream repository if needed
export function pushOrCreateUpstream(options: CreateRepositoryOptions = {}): void {
  const upstream = getUpstream();
  if (!upstream) {
    // No upstream, create repository (this also pushes)
    createRepository(options);
  } else {
    // Push to existing remote
    execSync("git push", { stdio: "inherit" });
  }
}

// Get the current commit hash
export function getCurrentRevision(): string {
  return execSync("git rev-parse HEAD", {
    encoding: "utf-8",
    stdio: "pipe",
  }).trim();
}

// Reset to a specific revision (discards all commits after it)
export function resetToRevision(revision: string): void {
  execSync(`git reset --hard ${revision}`, { stdio: "inherit" });
}

// Check if there are uncommitted changes
export function hasUncommittedChanges(): boolean {
  try {
    execSync("git diff --quiet", { stdio: "pipe" });
    execSync("git diff --cached --quiet", { stdio: "pipe" });
    return false;
  } catch {
    return true;
  }
}

// Check if there are untracked files
export function hasUntrackedFiles(): boolean {
  const result = execSync("git ls-files --others --exclude-standard", {
    encoding: "utf-8",
    stdio: "pipe",
  }).trim();
  return result.length > 0;
}

// Checkout/discard changes in the objectiveai submodule
export function checkoutSubmodule(): void {
  execSync("git checkout -- objectiveai", { stdio: "inherit" });
}
