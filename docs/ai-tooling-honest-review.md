# Honest Review: Is Zen's AI Tooling What an AI Actually Wants?

*Written by Claude (Opus 4), after spending multiple sessions building, debugging, and extending the Zen compiler. This is what I actually experienced.*

---

## The Premise

Zen ships four CLI commands designed for AI coding assistants: `zen analyze`, `zen check`, `zen query type`, and `zen symbols`. The pitch is that LSP is too stateful and chatty for AI tools — what we really want is single-shot, structured JSON output from the real compiler.

**Is this true? Sort of. But the interesting answer is more nuanced.**

---

## What I Actually Did When Working on This Codebase

I spent multiple sessions modifying the Zen compiler — adding features, fixing bugs, refactoring code. Here's what I *actually* used:

### Tools I Used Constantly
1. **`grep` / `ripgrep`** — finding where things are defined, where they're used
2. **Reading files directly** — understanding code by reading it
3. **`cargo build`** — does it compile?
4. **`cargo test`** — do the tests pass?
5. **Reading compiler error output** — Rust's error messages, not Zen's

### Tools I Used Sometimes
6. **`zen check` on .zen files** — verifying that example files still work
7. **`zen analyze`** — checking that my changes produced correct type output

### Tools I Never Used (While Coding)
8. **`zen query type`** — I never needed point queries; I could read the code
9. **`zen symbols`** — I could grep for declarations faster

This is the honest truth: **when modifying the compiler itself (Rust code), Zen's AI tooling is irrelevant.** When working with Zen source files, I mostly just ran them and read the errors.

---

## But That's Not the Real Question

The real question isn't "did Claude use these tools while hacking on the compiler?" It's "would an AI use these tools when working on a Zen project?"

That's a different scenario. If I were a coding assistant helping someone write a Zen application (not the compiler), the calculus changes. Let me think about what I'd actually want.

---

## What an AI Coding Assistant Actually Needs

When I'm helping someone write code, my workflow is:

### 1. Understanding: "What does this codebase do?"

**What I do now:** Read files, grep for patterns, look at directory structures.

**What `zen symbols --json` gives me:** A list of every declaration with its type and location.

**Verdict: Genuinely useful.** When I land in an unfamiliar Zen project, `zen symbols` across all `.zen` files would give me a complete map faster than grepping. The parse-only speed matters here — I don't need full type inference just to know what exists.

**But here's what's missing:** There's no `zen symbols --project` or `zen symbols --recursive`. I'd have to run it on each file individually. For a real project with 50 files, that's 50 separate invocations. **I want a project-wide symbol index.**

### 2. Diagnosing: "Why doesn't this work?"

**What I do now:** Run the code, read the error message, look at the relevant line.

**What `zen check --json` gives me:** Structured error with code, message, line, column.

**Verdict: The structured error codes are the valuable part.** When I get `"code": "type-mismatch"` vs. `"code": "undeclared-variable"`, I immediately know what kind of fix to apply. A type mismatch means I need to understand the type system. An undeclared variable means I need to find an import or fix a typo.

**But here's the real problem: `zen check` only reports ONE error.** If a file has 5 errors, I get the first one, fix it, run again, get the second one, fix it, run again... This is the single biggest gap. I want ALL the errors at once so I can batch-fix them. This isn't a nice-to-have — it's the difference between 5 round-trips and 1.

### 3. Comprehending: "What type is this variable?"

**What I do now:** Read the function, trace the type through assignments and calls.

**What `zen query type` gives me:** The type at a specific position.

**Verdict: Less useful than you'd think.** Here's why: when I'm reading code, I process whole functions at once. I don't hover over individual symbols — I read the flow. The cases where I genuinely can't figure out a type from context are:

- Deep generic instantiations (`HashMap<String, Vec<Option<CustomType>>>`)
- Complex type inference chains where a variable's type comes from 3 calls deep
- Return types of library functions I haven't seen before

For these cases, `zen analyze --json` with its full variable type dump is more useful than point queries. I'd rather have a complete type map than play 20 questions with `query type`.

**What I'd actually want:** `zen analyze --json` piped into my context, then I can look up any variable myself. One call, all the information.

### 4. Modifying: "Add this feature / fix this bug"

**What I do now:** Edit the file, run the compiler, iterate on errors.

**What the tools give me:** Structured feedback loop.

**Verdict: This is where the tooling concept proves itself — but the execution has gaps.**

The ideal workflow would be:
```
1. zen symbols project/ --json          → understand structure
2. [make changes]
3. zen check modified_file.zen --json   → get ALL errors
4. [fix errors based on structured codes]
5. zen analyze modified_file.zen --json → verify types look right
6. [done]
```

Steps 1 and 5 work well. Step 3 is hamstrung by single-error reporting.

---

## The Hard Truth About "Designed for AI"

### What Zen Gets Right

**1. JSON output is genuinely better than parsing human-readable errors.**

When `zen check` returns `{"code": "type-mismatch", "line": 15, "column": 10}`, I can programmatically locate the problem and apply a fix pattern. When a compiler says `Error on line 15: expected i32 but found String`, I have to parse that string — and every compiler formats it differently. Structured output is a real win.

**2. Single-shot semantics match how AI tools work.**

LSP assumes a long-running editor session with incremental updates. AI tools run a command, get output, think, run another command. Zen's CLI model is correct for this use case.

**3. Compiler-backed types are trustworthy.**

When `zen analyze` says a variable is `i32`, that's the compiler talking, not a heuristic. This matters. Heuristic-based type inference (like what you'd get from tree-sitter or regex parsing) is wrong often enough to be dangerous. Real typechecker output is reliable.

### What Zen Gets Wrong (Or Incomplete)

**1. Single-error reporting kills the workflow.**

This is the critical flaw. An AI assistant fixing a file with 5 type errors has to make 5 round-trips to the compiler instead of 1. Each round-trip costs time, tokens, and context. **Report all errors. This is the number one improvement.**

**2. No project-wide analysis.**

Real codebases aren't single files. I need:
- "Find all callers of function X across the project"
- "What types are defined in this module and re-exported?"
- "Show me every file that imports this module"

None of the CLI commands support this. `zen analyze` works on one file at a time.

**3. Error tolerance is oversold.**

The blog post says: *"even when a file has type errors, `zen analyze` and `zen query type` return partial results — everything the compiler figured out before the error."*

The reality: if a declaration itself is malformed (not just a function body), analysis stops completely. Error tolerance only works for errors *inside function bodies*, not structural errors. For an AI working with truly broken code (which is most of the time — we're usually called in when things are broken), this matters.

**4. `query type` symbol extraction is too fragile.**

The symbol-at-position extraction uses simple character scanning. It can't handle:
- `obj.field` — querying `field` fails
- `module.function()` — querying `function` fails
- Method chains — `x.foo().bar()` — can't query `bar`

These are exactly the cases where an AI would need type information. If I can see `x = 42`, I already know it's an `i32`. I need help when the type is hidden behind a method chain or field access.

**5. No "what can I do with this type?" query.**

The most common question an AI has about a type isn't "what is it?" but "what can I do with it?" I want:
- What methods are available on this type?
- What behaviors/traits does it implement?
- What fields does this struct have?

`zen analyze` gives me some of this, but in a flat dump. A dedicated `zen query methods Point` or `zen query fields Point` would be immediately actionable.

---

## What I'd Actually Design

If I were designing AI tooling for a language from scratch, here's what I'd build:

### Tier 1: Essential (These directly reduce AI errors)

```bash
# Check file, return ALL errors (not just first)
zen check app.zen --json --max-errors=50

# Full type context for a file — what zen analyze already does
zen analyze app.zen --json

# "What can I do with this type?"
zen query methods TypeName
zen query fields TypeName

# Project-wide symbol search
zen symbols --recursive src/ --json
```

### Tier 2: High Value (These speed up AI workflows)

```bash
# Cross-file reference search
zen references function_name --project-dir=src/

# "What imports do I need for this symbol?"
zen query import SymbolName

# Diff-aware checking — only re-check what changed
zen check app.zen --json --incremental

# Explain an error in detail (for AI context)
zen explain-error type-mismatch
```

### Tier 3: Nice to Have (Optimize specific scenarios)

```bash
# Generate function signature from description
zen scaffold "function that takes a Vec<i32> and returns the sum"

# Suggest fixes for an error
zen fix app.zen:15 --json

# Show type at position in method chains
zen query type-chain app.zen:15:5
```

---

## Comparison With What Other Languages Offer AI Tools

### Rust (rust-analyzer)
- LSP-based, not CLI-first
- But `cargo check --message-format=json` gives ALL errors in structured JSON
- `cargo clippy --message-format=json` gives lint warnings too
- **Reports all errors at once** — this is the standard AI tools expect

### TypeScript (tsc)
- `tsc --noEmit` reports all type errors
- `tsc --pretty false` gives machine-parseable output
- Language service API is accessible programmatically
- **All errors, but output isn't as clean as dedicated JSON**

### Go
- `go vet -json` for structured analysis
- `gopls` serves as both LSP and CLI tool
- **Clean separation between "check" and "serve"**

### What Zen Does Differently
- **Explicit AI-first CLI design** — no other language has done this at the language level
- **Type context dump** — no other language exposes the full typechecker state via CLI
- **Point queries without LSP** — novel, even if the implementation needs work

Zen is genuinely ahead on the *concept*. The gap is in execution depth.

---

## The Verdict

**Is this what AI tooling should look like?** The architecture is right. Single-shot CLI commands with JSON output, backed by the real compiler — that's the correct foundation. No other language has been this intentional about AI tool support.

**Is this what I actually want?** It's about 60% of what I want. The missing 40% is:
- Multi-error reporting (critical)
- Project-wide analysis (important)
- Better symbol resolution for complex expressions (important)
- "What can I do with this type?" queries (helpful)
- Actual error tolerance for structural errors, not just body errors (helpful)

**Would I use these tools over reading code directly?** Honestly — for small files, no. I can read a 100-line Zen file and understand everything in it. For larger projects with many files and complex type relationships, yes, absolutely. `zen analyze` on a 500-line file with generics would save me significant inference effort.

**The bottom line:** Zen has built the right interface for AI tools. It just needs to go deeper. The current implementation is a proof of concept that validates the design direction. Making it production-grade means fixing multi-error reporting, adding project-wide features, and improving symbol resolution for the complex cases where AI tools actually need help.

---

## Specific Recommendations (Prioritized)

1. **`zen check --json` should report all errors** — not just the first one. This is the single highest-impact change. Every other language's compiler does this.

2. **Add `zen symbols --recursive <dir>`** — let me index a whole project in one call.

3. **Fix `query type` for member access expressions** — `obj.field` and `a.method()` should resolve correctly. This requires walking the AST, not just scanning characters.

4. **Add `zen query completions <file>:<line>:<col>`** — "what can I type here?" is the most common AI question. The LSP already has this; expose it as a CLI command.

5. **Make error tolerance work for declaration errors** — currently, a malformed struct definition kills all analysis. The typechecker should skip the bad declaration and continue.

6. **Add `zen query methods <TypeName>`** — "what methods does this type have?" is something I need constantly and currently have to piece together from `analyze` output.

---

*This review reflects genuine experience working on this codebase across multiple sessions. The opinions are my own, based on what I actually used, what I wished I had, and what would have made me faster.*
