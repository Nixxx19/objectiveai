# Vector Completions Endpoint Method Mismatch

**Issue Type:** Architectural Clarification Required
**Priority:** High (blocks deployment confidence)
**Date:** February 9, 2026
**Flagged By:** Comprehensive audit cross-referencing backend and SDK implementations

## Summary

There is an inconsistency between the backend API route registration and the TypeScript SDK method calls for two vector completions endpoints. The backend registers them as POST, but the SDK calls them with GET.

## Affected Endpoints

1. `/vector/completions/{id}` - Retrieve specific vector completion by ID
2. `/vector/completions/cache` - Query vector completions cache

## Current Implementation

### Backend (objectiveai-api/src/main.rs)

Lines 296-327:
```rust
// POST /vector/completions/{id}
.route("/vector/completions/:id", axum::routing::post(/* handler */))  // Line 298

// POST /vector/completions/cache
.route("/vector/completions/cache", axum::routing::post(/* handler */))  // Line 314
```

Both endpoints are registered using `axum::routing::post()`.

### TypeScript SDK (objectiveai-js/src/vector/completions/http.ts)

Line 54:
```typescript
retrieve(client: ObjectiveAI, id: string): Promise<Retrieve.Response> {
  return client.get_unary<Retrieve>(Retrieve, `/vector/completions/${id}`);
}
```

The SDK uses `client.get_unary()` which sends GET requests.

## Documentation Note

From CLAUDE.md line 801:
> Note for Ronald: API docs show `/vector/completions/{id}` and `/vector/completions/cache` as GET (matching SDK `get_unary`), but `objectiveai-api/src/main.rs` registers them as POST. Docs match the SDK — verify if this is intentional.

## Question for Ronald

**Which is correct?**

1. **Option A:** Backend should be GET
   - Change `axum::routing::post()` to `axum::routing::get()` in main.rs lines 298 and 314
   - SDK and docs are already correct

2. **Option B:** SDK should be POST
   - Change `client.get_unary()` to `client.post_unary()` in TypeScript SDK
   - Update API documentation to reflect POST methods

## Impact

- This mismatch affects any code trying to retrieve vector completion results or query the cache
- Currently unclear which implementation is correct
- Needs resolution before production deployment

## Verification Needed

Please verify:
1. What is the intended HTTP method for these endpoints?
2. Are there semantic reasons one method is preferred over the other?
3. Should both implementations be aligned, or is there a deeper architectural reason for the difference?

## Related Files

- Backend: [objectiveai-api/src/main.rs](objectiveai-api/src/main.rs#L296-L327)
- SDK: [objectiveai-js/src/vector/completions/http.ts](objectiveai-js/src/vector/completions/http.ts#L54)
- Docs: [CLAUDE.md line 801](CLAUDE.md#L801)

---

## ✅ RESOLVED (February 9, 2026)

**Resolution:** Documentation bug corrected. The backend and SDK both correctly use POST methods. The issue was in the web app documentation showing GET instead of POST.

### Changes Made

1. **SDK was already correct** - Uses `client.post_unary()` for both endpoints (commit e8a60e9)
2. **Documentation fixed:**
   - Updated `DocsSidebar.tsx` to show POST methods (lines 219, 226)
   - Updated `app/information/page.tsx` to show POST methods (lines 169, 170)
   - Moved doc pages from `/docs/api/get/` to `/docs/api/post/` directories
3. **Verification:** Full build succeeded with no TypeScript errors
4. **Confirmation:** All 34 documented API endpoints now match backend implementation

### Root Cause

The issue description in this file was based on outdated code. The SDK was fixed to use POST in commit e8a60e9, but the documentation still showed GET, causing confusion.

**Status:** CLOSED - All documentation now correctly reflects POST methods matching backend implementation.
