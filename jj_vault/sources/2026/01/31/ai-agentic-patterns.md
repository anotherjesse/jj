---
id: src_01KGAHWXK99F9EMPAY5CMPCB43
title: ai-agentic-patterns
ingested_at: 2026-01-31T17:34:03.881666+00:00
original_path: /Users/jesse/cto/references/ai-agentic-patterns.md
tags: []
processing_status: complete
content_hash: sha256:9be8bdc1f32b9575375dbbe12677093b4f18c8ef58647e6fa1b82df1a0125dd1
---
# AI & Agentic Patterns - Reference Collection

Curated articles on working effectively with AI agents, context management, and verification patterns.

---

## Back Pressure & Feedback Loops

### Don't Waste Your Back Pressure
**Source:** https://banay.me/dont-waste-your-backpressure/
**Author:** Banay

**Core insight:** Projects that provide agents with automated feedback on quality and correctness can push them to work on longer horizon tasks. This "back pressure" helps agents identify mistakes as they progress.

**Key points:**
- Without automated feedback, you spend YOUR back pressure (human review time) on trivial errors like missed imports
- Give agents tools to run builds, read feedback, and self-correct
- Typed languages with expressive type systems create natural back pressure
- Languages with excellent error messages (Rust, Elm, Python) work better - errors feed directly back to LLM
- Screenshot/browser automation (Playwright, Chrome DevTools) lets agents verify UI without human involvement
- "Loop agents until they have stamped out all the inconsistencies"

**Related:** RALPH loops (https://ghuntley.com/ralph/), Factory.ai Agent Readiness

---

## Practitioner Workflows

### Shipping at Inference-Speed
**Source:** https://steipete.me/posts/2025/shipping-at-inference-speed
**Author:** Peter Steinberger
**Date:** December 2025

**Core insight:** With GPT 5.2/Codex, the bottleneck is inference time and hard thinking, not coding. Most software doesn't require hard thinking.

**Key workflow patterns:**
- Work on 3-8 projects simultaneously
- Use queueing feature extensively - add ideas to pipeline as they come
- Rarely revert - ask model to change it instead
- Commit directly to main (when working solo)
- Start conversations with model, create plan together, then say "build"
- Don't read much code anymore - watch the stream, know the structure

**Model observations:**
- Codex reads files for 10-15 minutes before writing (increases success rate)
- Opus is more eager (good for small edits, bad for large features)
- GPT 5.2 knowledge cutoff is ~5 months newer than Opus (significant for latest tools)

**Context management:**
- Maintain docs folder per project with subsystem documentation
- Cross-reference projects: "look at ../vibetunnel and do the same"
- No need to restart sessions with GPT 5.2 - context performance stays good

**Infrastructure:**
- Start everything as CLI first - agents can call it, close the loop
- Automate everything via skills
- Multiple Macs with Jump Desktop for parallel work

**Config (codex):**
```toml
model = "gpt-5.2-codex"
model_reasoning_effort = "high"
tool_output_token_limit = 25000
model_auto_compact_token_limit = 233000
```

---

## Codebase Readiness

### Introducing Agent Readiness
**Source:** https://factory.ai/news/agent-readiness
**Author:** Factory.ai
**Date:** January 2026

**Core insight:** Uneven agent results are usually the codebase's fault, not the model's. A codebase with poor feedback loops will defeat any agent.

**Eight technical pillars:**
1. Pre-commit hooks (fast feedback vs 10-min CI wait)
2. Environment documentation
3. Build process documentation
4. Testing infrastructure
5. CI/CD configuration
6. Security scanning
7. Code ownership (CODEOWNERS)
8. Branch protection

**Five maturity levels:**
- Level 1: Basic
- Level 2: Foundational
- Level 3: Agent-ready (target for most repos)
- Level 4: Comprehensive
- Level 5: Exemplary

**Usage:**
- CLI: `/readiness-report` in Factory Droid
- Dashboard: app.factory.ai/analytics/readiness
- API: Programmatic access for CI/CD integration

**Key metric:** "80% of our active repos are agent-ready" is more actionable than average scores.

---

## Creativity & Exploration

### Lluminate: Creative Exploration with Reasoning LLMs
**Source:** https://www.joelsimon.net/lluminate
**Author:** Joel Simon
**Date:** December 2025
**Code:** https://github.com/joel-simon/lluminate

**Core insight:** LLMs default to homogeneous outputs. Evolutionary pressure + formalized creative strategies enables sustained open-ended exploration.

**The problem:** Ask for "an interesting shader" and you get saturated colors and rotating spirals every time. LLMs converge to average solutions.

**Algorithm:**
1. Generate population summary for context
2. Inject random creative strategy
3. Evolve via mutation/crossover
4. Embed outputs and measure novelty (cosine distance to k nearest neighbors)
5. Select most diverse for next generation

**Creative strategies tested:**
- Oblique Strategies (Brian Eno)
- Conceptual Blending
- Assumption Reversal
- Cross-Domain Transfer
- SCAMPER Transformation
- Distance Association

**Key findings:**
- Variation (mutating existing) beats creation (generating new from scratch)
- Crossover amplifies novelty most
- Novelty correlates with complexity (longer code = more novel territory)
- Higher reasoning levels don't significantly increase diversity
- Different strategies work better for different domains

---

## Context & Reading

### How To Use AI for the Ancient Art of Close Reading
**Source:** https://www.fast.ai/posts/2026-01-21-reading-LLMs/
**Authors:** Jeremy Howard, Eric Ries, Johno Whitaker (fast.ai)
**Date:** January 2026

**Core insight:** Close reading with LLMs - pausing after paragraphs to ask clarifying questions, make connections, go down rabbit holes. "One of the absolute best reading experiences I've ever had."

**Process:**
1. Convert PDFs to Markdown
2. Generate chapter summaries as LLM context
3. Instruct LLM not to give spoilers
4. Read through full text, asking questions as you go
5. Generate conversation overviews at chapter end for next chapter's context
6. Optional: LLM asks comprehension questions
7. Optional: Create Anki cards via fastanki

**Benefits:**
- Go down rabbit holes of interest (discovered 4 of Jack Welch's 13 failed mentees were Boeing CEOs)
- Ask for counterexamples when skeptical
- Personalize principles to your situation
- Spaced repetition integration

**Key quote:** "It's like the architect sharpening his pencils... that little investment up front makes it a very different tool to the vanilla case."

**Tools:** SolveIt platform (https://solve.it.com/), fastanki library

---

## Tool Architecture

### Code Mode: The Better Way to Use MCP
**Source:** https://blog.cloudflare.com/code-mode/
**Author:** Cloudflare
**Date:** September 2025

**Core insight:** LLMs are better at writing code to call MCP than at calling MCP directly. Convert MCP tools into a TypeScript API and ask the LLM to write code that calls it.

**Why it works:**
- LLMs have enormous amounts of real-world TypeScript in training
- Only a small set of contrived tool-call examples in training
- Multi-step tasks: code can chain calls without feeding outputs back through neural network

**Implementation:**
- MCP schema → TypeScript API with doc comments
- Agent gets one tool: execute TypeScript
- Code runs in V8 isolate sandbox (no containers, millisecond startup)
- Sandbox has no internet access - only bindings to MCP servers
- API keys never exposed to generated code

**Key architectural points:**
- Isolates are far more lightweight than containers
- Bindings approach cleaner than network filtering
- Worker Loader API for on-demand isolate creation

```typescript
import { codemode } from "agents/codemode/ai";

const {system, tools} = codemode({
  system: "You are a helpful assistant",
  tools: { /* tool definitions */ },
})
```

---

## Cross-Cutting Themes

1. **Context management is everything** - summaries of summaries, docs folders, JIT loading
2. **Automated feedback > human review** - back pressure, type systems, screenshots
3. **Probabilistic thinking** - run many experiments, not one golden attempt
4. **Start with CLI** - agents can call it, close the loop, verify output
5. **Less scaffolding as models improve** - build SDLC around new physics, not hand-holding

---

*Last updated: 2026-01-23*
