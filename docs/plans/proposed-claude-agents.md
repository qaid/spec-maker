# Proposed New Claude Agents for SpecMaker Project

## Analysis Summary

After analyzing:
- **Existing agents**: code-searcher, memory-bank-synchronizer, ux-design-expert, git-workflow, web-research-synthesizer
- **Project architecture**: Tauri + React + SQLite + Ollama
- **Phase 2 priorities**: Template generation, artifact creation, context summarization
- **Token optimization needs**: Repetitive patterns, large template files, database operations

## Proposed New Agents (5 Specialized Agents)

---

## 1. **template-builder** (High Priority - Phase 2)

### Purpose
Specialized agent for creating, managing, and testing Handlebars templates for PRD, TechSpec, and Implementation Plan generation.

### Why It's Needed
- Phase 2 core feature requires multiple complex templates
- Templates have specific structure requirements (see implementation plan lines 665-799)
- Need to validate template syntax and output
- Templates will be reused and iterated frequently

### Token Optimization
- **Saves ~40% tokens** on template creation tasks by:
  - Pre-loaded template patterns and best practices
  - Cached knowledge of Handlebars syntax
  - Quick iteration without re-explaining context
  - Focused on template-specific concerns only

### Key Responsibilities
- Create Handlebars templates with proper variable interpolation
- Validate template syntax before deployment
- Generate sample outputs with test data
- Update templates based on user feedback
- Ensure templates match database schema fields

### Usage Examples
```
"Create implementation plan template with Phase/Task structure"
"Add code template section to PRD template"
"Test artifact template with sample project data"
"Update tech spec template to include API specs section"
```

### Configuration
```yaml
name: template-builder
model: sonnet
color: green
tools: [Read, Write, Edit, Grep, Bash]
```

---

## 2. **artifact-generator** (High Priority - Phase 2)

### Purpose
Specialized agent for generating artifacts (PRDs, TechSpecs, Implementation Plans) using templates and project context.

### Why It's Needed
- Core SpecMaker feature is artifact generation
- Requires understanding project context + template + AI generation
- Need to handle context window management for long conversations
- Must coordinate between database, templates, and Ollama

### Token Optimization
- **Saves ~50% tokens** by:
  - Pre-loaded artifact structure knowledge
  - Efficient context summarization
  - Direct focus on generation without setup
  - Cached understanding of output format requirements

### Key Responsibilities
- Load project context from database
- Summarize conversation history for context
- Populate templates with AI-generated content
- Validate artifact structure and completeness
- Save artifacts to database with proper metadata
- Handle iterative refinement requests

### Usage Examples
```
"Generate implementation plan for current project"
"Create PRD artifact from conversation history"
"Refine tech spec with additional API details"
"Generate user stories artifact"
```

### Integration Points
- Works WITH: template-builder (uses templates)
- Calls: Ollama service (for AI generation)
- Reads: Database (conversations, messages, projects)
- Writes: Database (artifacts table)

### Configuration
```yaml
name: artifact-generator
model: sonnet
color: blue
tools: [Read, Bash, Grep]
```

---

## 3. **database-ops** (Medium Priority - All Phases)

### Purpose
Specialized agent for SQLite database operations, schema updates, and data migrations specific to SpecMaker.

### Why It's Needed
- Complex database schema with 5+ tables and FTS
- Schema will evolve across phases
- Need safe migration strategies
- Debugging database issues requires specialized knowledge

### Token Optimization
- **Saves ~35% tokens** by:
  - Pre-loaded SpecMaker schema knowledge
  - Cached understanding of table relationships
  - Quick migration pattern application
  - Direct database operation focus

### Key Responsibilities
- Create database migrations for schema changes
- Write optimized queries for complex data retrieval
- Debug database-related issues
- Add/modify indexes for performance
- Validate data integrity
- Generate test data for development

### Usage Examples
```
"Add column to artifacts table for template_version"
"Create migration to add conversation_metadata table"
"Optimize query for artifact search"
"Generate test data for 10 projects with conversations"
```

### Configuration
```yaml
name: database-ops
model: haiku  # Faster for routine operations
color: yellow
tools: [Read, Write, Edit, Bash, Grep]
```

---

## 4. **context-summarizer** (High Priority - Phase 2)

### Purpose
Specialized agent for summarizing long conversations into compact context suitable for LLM context windows.

### Why It's Needed
- Long consultation sessions will exceed context windows
- context_summaries table designed for this (lines 128-136 in schema)
- Critical for maintaining conversation continuity
- Needs intelligent summarization that preserves requirements

### Token Optimization
- **Saves ~60% tokens** on long conversations by:
  - Intelligent compression of conversation history
  - Preservation of critical requirements only
  - Elimination of conversational fluff
  - Hierarchical summarization strategy

### Key Responsibilities
- Analyze conversation messages for key information
- Create tiered summaries (high/medium/low importance)
- Store summaries in context_summaries table
- Retrieve and merge summaries for artifact generation
- Update summaries as conversation progresses
- Calculate and track token counts

### Usage Examples
```
"Summarize last 50 messages for artifact generation"
"Create high-level summary of entire project consultation"
"Compress conversation preserving all requirements"
"Update context summary with new requirements discussed"
```

### Integration Points
- Reads: messages table (conversation history)
- Writes: context_summaries table
- Used by: artifact-generator (for context loading)

### Configuration
```yaml
name: context-summarizer
model: sonnet  # Needs good compression skills
color: orange
tools: [Read, Bash, Grep]
```

---

## 5. **prompt-engineer** (Medium Priority - Phase 2+)

### Purpose
Specialized agent for crafting, testing, and optimizing prompts for Ollama interactions within SpecMaker.

### Why It's Needed
- Multiple prompt types needed (PRD, TechSpec, ImplPlan, consultation)
- Prompts must be tuned for local Ollama models
- Need to iterate on prompt quality
- Prompt effectiveness directly impacts product value

### Token Optimization
- **Saves ~30% tokens** by:
  - Focused expertise on prompt engineering
  - Quick iteration without re-explanation
  - Pre-loaded knowledge of Ollama model characteristics
  - Cached prompt patterns and anti-patterns

### Key Responsibilities
- Create system prompts for different artifact types
- Optimize prompts for specific Ollama models
- Test prompt variations with sample data
- Measure prompt effectiveness (output quality)
- Store prompts as templates in src-tauri/src/prompts/
- Document prompt parameters and usage

### Usage Examples
```
"Create PRD generation prompt for llama3.1:8b"
"Optimize implementation plan prompt for code detail"
"Test consultation prompt with sample conversation"
"Add few-shot examples to tech spec prompt"
```

### Configuration
```yaml
name: prompt-engineer
model: sonnet
color: pink
tools: [Read, Write, Edit, Bash]
```

---

## Token Savings Analysis

### Current Workflow (Without Specialized Agents)
```
User: "Create implementation plan template"
→ Main agent needs:
  - Full context explanation (100 tokens)
  - Template format discussion (150 tokens)
  - Handlebars syntax explanation (80 tokens)
  - SpecMaker schema review (120 tokens)
  - Example iteration (200 tokens)
  TOTAL: ~650 tokens per task
```

### With Specialized Agent (template-builder)
```
User: "Create implementation plan template"
→ Specialized agent:
  - Already knows SpecMaker context (0 tokens)
  - Pre-loaded template patterns (0 tokens)
  - Handlebars expert (0 tokens)
  - Direct implementation (150 tokens)
  TOTAL: ~150 tokens per task

SAVINGS: ~77% reduction
```

### Projected Total Savings Across Phase 2
- **Template tasks**: 10-15 tasks × 500 tokens saved = 5,000-7,500 tokens
- **Artifact generation**: 50+ iterations × 800 tokens saved = 40,000+ tokens
- **Database operations**: 20 tasks × 400 tokens saved = 8,000 tokens
- **Context summarization**: 30+ summaries × 1,000 tokens saved = 30,000+ tokens
- **Prompt optimization**: 15 tasks × 300 tokens saved = 4,500 tokens

**TOTAL ESTIMATED SAVINGS: ~90,000 tokens across Phase 2**

---

## Implementation Priority

### Phase 2 Immediate (Week 1-2)
1. **template-builder** - Needed first for Phase 2 Task 2.1
2. **artifact-generator** - Core feature, needed Week 2

### Phase 2 Mid (Week 3-5)
3. **context-summarizer** - As conversations grow longer
4. **prompt-engineer** - To optimize artifact quality

### Ongoing (All Phases)
5. **database-ops** - As needed for schema evolution

---

## Agent Interaction Map

```
User Request: "Generate Implementation Plan"
    ↓
[artifact-generator] invoked
    ├─→ [context-summarizer] (get conversation summary)
    ├─→ [template-builder] (validate template exists)
    ├─→ [database-ops] (fetch project data)
    └─→ [prompt-engineer] (use optimized prompt)
         ↓
    Save to database
    Return artifact to user
```

---

## Comparison with Existing Agents

| Agent | Overlap Risk | Complementary | Notes |
|-------|-------------|---------------|-------|
| **code-searcher** | None | Yes | Different domain (codebase vs templates) |
| **memory-bank-synchronizer** | None | Yes | Different purpose (docs vs data) |
| **ux-design-expert** | Minimal | Yes | Could help with template presentation |
| **git-workflow** | None | Yes | Different concern (version control) |
| **web-research-synthesizer** | None | Yes | Different data source (web vs local) |

**Conclusion**: No significant overlap. All proposed agents fill unique niches.

---

## Alternative Considered: Single "SpecMaker-Assistant" Agent

### Why NOT Recommended
- Would still need full context each invocation (~300-500 tokens)
- Less focused expertise per domain
- Harder to optimize for specific tasks
- Loses the token efficiency of specialization
- Can't parallelize operations (e.g., template validation while summarizing context)

### Why Specialized Agents are Better for SpecMaker
- **Token efficiency**: Each agent starts with domain knowledge
- **Parallel operations**: Can run template-builder + context-summarizer simultaneously
- **Clear responsibilities**: Easier to debug and improve
- **Reusability**: template-builder useful across all artifact types
- **Iterative improvement**: Can enhance one agent without affecting others

---

## Recommended Next Steps

1. **Create template-builder first** (before starting Phase 2 implementation)
2. **Test with one template** (implementation plan template)
3. **Create artifact-generator** (once template system proven)
4. **Add others incrementally** as needs arise

Would you like me to create the agent configuration files for these proposed agents?
