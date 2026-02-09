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

**Action Required:** Please review and provide guidance on which implementation should be corrected.
