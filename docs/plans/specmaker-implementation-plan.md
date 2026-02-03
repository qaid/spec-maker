# SpecMaker Implementation Plan

> **Implementation Mode**: Claude Code as implementer, user as orchestrator
> **This document serves as the specification for Claude Code to build SpecMaker**

## Executive Summary

Build **SpecMaker**, a MacOS desktop app that generates detailed PRDs and AI-coding-agent-ready implementation plans using local AI (Ollama). The tool acts as an intelligent product consultant, producing file-by-file implementation specifications suitable for Claude Code, Cursor, and similar AI coding agents.

### Key Constraint
The user will orchestrate development using Claude Code. This plan must be detailed enough for Claude Code to implement each component correctly without ambiguity.

---

## Technology Stack (Final)

| Layer | Technology | Rationale |
|-------|------------|-----------|
| **Framework** | Tauri 2.0 | Lightweight, native performance, Rust backend |
| **Frontend** | React 18 + TypeScript | Mature ecosystem, type safety |
| **UI Components** | shadcn/ui + Tailwind CSS | Professional look, customizable |
| **State Management** | Zustand | Simple, performant, TypeScript-native |
| **Backend** | Rust | Tauri requirement, excellent for LLM integration |
| **Database** | SQLite (rusqlite) | Local-first, no server needed |
| **LLM** | Ollama | Already installed, local, streaming support |
| **Templates** | Handlebars | Proven templating, good Rust support |
| **Markdown** | pulldown-cmark (Rust), react-markdown (TS) | Fast parsing, good rendering |

---

## Reordered Implementation Phases

> **Priority Change**: File-level implementation plans (the core value) moved to Phase 2 instead of Phase 3

### Phase 1: Foundation (Weeks 1-4)
Core infrastructure: Tauri shell, Ollama connection, basic chat, SQLite persistence

### Phase 2: Core Value - Implementation Plan Generation (Weeks 5-10) ⭐ PRIORITY
The killer feature: generating detailed, AI-coding-agent-ready implementation plans

### Phase 3: Enhanced Consultation & PRD (Weeks 11-16)
Multi-turn consultation, industry-aware questions, full PRD/TechSpec generation

### Phase 4: Polish & Production (Weeks 17-22)
Export formats, settings, search, macOS integration, quality checks

---

## Phase 1: Foundation

### Task 1.1: Initialize Tauri Project

**Command for Claude Code:**
```bash
npm create tauri-app@latest spec-maker -- --template react-ts
cd spec-maker
npm install
```

**Post-initialization modifications:**
1. Update `src-tauri/tauri.conf.json` with app metadata
2. Configure Tailwind CSS
3. Install shadcn/ui
4. Set up project structure

**Acceptance Criteria:**
- [ ] `npm run tauri dev` launches empty window titled "SpecMaker"
- [ ] Tailwind CSS working (test with colored div)
- [ ] shadcn/ui Button component renders correctly

---

### Task 1.2: Database Schema & Setup

**File: `src-tauri/src/database/schema.sql`**

```sql
-- SpecMaker Database Schema v1.0

-- Projects: Top-level container for a product specification
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    industry TEXT,
    target_audience TEXT,
    status TEXT NOT NULL DEFAULT 'ideation'
        CHECK (status IN ('ideation', 'consultation', 'generating', 'review', 'complete')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Conversations: Chat sessions within a project
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    phase TEXT NOT NULL DEFAULT 'initial_analysis'
        CHECK (phase IN ('initial_analysis', 'consultation', 'context_building', 'generation', 'refinement')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Messages: Individual chat messages
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    metadata TEXT, -- JSON: {token_count, model, phase_context}
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Artifacts: Generated documents (PRD, TechSpec, Stories, ImplPlan)
CREATE TABLE artifacts (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    artifact_type TEXT NOT NULL
        CHECK (artifact_type IN ('prd', 'tech_spec', 'user_stories', 'implementation_plan')),
    title TEXT NOT NULL,
    content TEXT NOT NULL, -- Markdown content
    version INTEGER NOT NULL DEFAULT 1,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'approved', 'archived')),
    metadata TEXT, -- JSON: {sections, word_count, generation_params}
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Context Summaries: Compressed conversation history for long sessions
CREATE TABLE context_summaries (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    summary TEXT NOT NULL,
    message_range_start TEXT NOT NULL, -- First message ID summarized
    message_range_end TEXT NOT NULL,   -- Last message ID summarized
    token_count INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for performance
CREATE INDEX idx_projects_status ON projects(status);
CREATE INDEX idx_projects_updated ON projects(updated_at DESC);
CREATE INDEX idx_conversations_project ON conversations(project_id);
CREATE INDEX idx_messages_conversation ON messages(conversation_id);
CREATE INDEX idx_messages_created ON messages(created_at);
CREATE INDEX idx_artifacts_project ON artifacts(project_id);
CREATE INDEX idx_artifacts_type ON artifacts(artifact_type);

-- Full-text search for artifacts
CREATE VIRTUAL TABLE artifacts_fts USING fts5(
    title,
    content,
    content='artifacts',
    content_rowid='rowid'
);

-- FTS sync triggers
CREATE TRIGGER artifacts_ai AFTER INSERT ON artifacts BEGIN
    INSERT INTO artifacts_fts(rowid, title, content) VALUES (new.rowid, new.title, new.content);
END;

CREATE TRIGGER artifacts_ad AFTER DELETE ON artifacts BEGIN
    INSERT INTO artifacts_fts(artifacts_fts, rowid, title, content) VALUES('delete', old.rowid, old.title, old.content);
END;

CREATE TRIGGER artifacts_au AFTER UPDATE ON artifacts BEGIN
    INSERT INTO artifacts_fts(artifacts_fts, rowid, title, content) VALUES('delete', old.rowid, old.title, old.content);
    INSERT INTO artifacts_fts(rowid, title, content) VALUES (new.rowid, new.title, new.content);
END;
```

**File: `src-tauri/src/database/mod.rs`**

```rust
use rusqlite::{Connection, Result};
use std::path::PathBuf;
use tauri::AppHandle;

pub fn get_db_path(app: &AppHandle) -> PathBuf {
    let app_dir = app.path_resolver().app_data_dir().unwrap();
    std::fs::create_dir_all(&app_dir).unwrap();
    app_dir.join("specmaker.db")
}

pub fn init_database(app: &AppHandle) -> Result<Connection> {
    let db_path = get_db_path(app);
    let conn = Connection::open(&db_path)?;

    // Run schema
    conn.execute_batch(include_str!("schema.sql"))?;

    Ok(conn)
}
```

**Acceptance Criteria:**
- [ ] Database file created at `~/Library/Application Support/com.specmaker.app/specmaker.db`
- [ ] All tables created successfully
- [ ] FTS index working (test with sample data)

---

### Task 1.3: Ollama Service

**File: `src-tauri/src/services/ollama.rs`**

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

const OLLAMA_BASE_URL: &str = "http://localhost:11434";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            model: "llama3.1:8b".to_string(),
            temperature: 0.7,
            max_tokens: Some(4096),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    options: ChatOptions,
}

#[derive(Debug, Serialize)]
struct ChatOptions {
    temperature: f32,
    num_predict: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    message: ChatMessage,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    message: Option<ChatMessage>,
    done: bool,
}

pub struct OllamaService {
    client: Client,
    config: OllamaConfig,
}

impl OllamaService {
    pub fn new(config: OllamaConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Check if Ollama is running and the model is available
    pub async fn health_check(&self) -> Result<bool, String> {
        let url = format!("{}/api/tags", OLLAMA_BASE_URL);
        match self.client.get(&url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    // Check if our model is available
                    let models: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
                    let model_exists = models["models"]
                        .as_array()
                        .map(|arr| arr.iter().any(|m| {
                            m["name"].as_str().map(|n| n.starts_with(&self.config.model)).unwrap_or(false)
                        }))
                        .unwrap_or(false);
                    Ok(model_exists)
                } else {
                    Err(format!("Ollama returned status: {}", resp.status()))
                }
            }
            Err(e) => Err(format!("Cannot connect to Ollama: {}. Is it running?", e)),
        }
    }

    /// Generate a response (non-streaming)
    pub async fn generate(&self, messages: Vec<ChatMessage>) -> Result<String, String> {
        let url = format!("{}/api/chat", OLLAMA_BASE_URL);
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            stream: false,
            options: ChatOptions {
                temperature: self.config.temperature,
                num_predict: self.config.max_tokens,
            },
        };

        let resp = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let chat_resp: ChatResponse = resp.json().await.map_err(|e| e.to_string())?;
        Ok(chat_resp.message.content)
    }

    /// Generate a response with streaming (returns channel receiver)
    pub async fn generate_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<mpsc::Receiver<Result<String, String>>, String> {
        let url = format!("{}/api/chat", OLLAMA_BASE_URL);
        let (tx, rx) = mpsc::channel(100);

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            stream: true,
            options: ChatOptions {
                temperature: self.config.temperature,
                num_predict: self.config.max_tokens,
            },
        };

        let client = self.client.clone();

        tokio::spawn(async move {
            match client.post(&url).json(&request).send().await {
                Ok(resp) => {
                    let mut stream = resp.bytes_stream();
                    use futures_util::StreamExt;

                    while let Some(chunk) = stream.next().await {
                        match chunk {
                            Ok(bytes) => {
                                if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                                    for line in text.lines() {
                                        if let Ok(parsed) = serde_json::from_str::<StreamChunk>(line) {
                                            if let Some(msg) = parsed.message {
                                                let _ = tx.send(Ok(msg.content)).await;
                                            }
                                            if parsed.done {
                                                return;
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(Err(e.to_string())).await;
                                return;
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string())).await;
                }
            }
        });

        Ok(rx)
    }
}
```

**Acceptance Criteria:**
- [ ] `health_check()` returns `Ok(true)` when Ollama running with model
- [ ] `health_check()` returns descriptive error when Ollama not running
- [ ] `generate()` returns complete response for simple prompt
- [ ] `generate_stream()` yields tokens incrementally

---

### Task 1.4: Tauri Commands (IPC Bridge)

**File: `src-tauri/src/commands/mod.rs`**

```rust
pub mod ollama;
pub mod project;
pub mod conversation;
pub mod artifact;

pub use ollama::*;
pub use project::*;
pub use conversation::*;
pub use artifact::*;
```

**File: `src-tauri/src/commands/ollama.rs`**

```rust
use crate::services::ollama::{OllamaService, OllamaConfig, ChatMessage};
use tauri::{command, State, Window};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct OllamaState(pub Arc<Mutex<OllamaService>>);

#[command]
pub async fn check_ollama_status(
    state: State<'_, OllamaState>,
) -> Result<bool, String> {
    let service = state.0.lock().await;
    service.health_check().await
}

#[command]
pub async fn send_message(
    state: State<'_, OllamaState>,
    window: Window,
    messages: Vec<ChatMessage>,
) -> Result<String, String> {
    let service = state.0.lock().await;

    // Use streaming and emit events to frontend
    let mut rx = service.generate_stream(messages).await?;
    let mut full_response = String::new();

    while let Some(result) = rx.recv().await {
        match result {
            Ok(chunk) => {
                full_response.push_str(&chunk);
                // Emit chunk to frontend for real-time display
                window.emit("llm-chunk", &chunk).unwrap_or_default();
            }
            Err(e) => {
                window.emit("llm-error", &e).unwrap_or_default();
                return Err(e);
            }
        }
    }

    window.emit("llm-complete", &full_response).unwrap_or_default();
    Ok(full_response)
}

#[command]
pub async fn update_ollama_config(
    state: State<'_, OllamaState>,
    config: OllamaConfig,
) -> Result<(), String> {
    let mut service = state.0.lock().await;
    *service = OllamaService::new(config);
    Ok(())
}
```

**Acceptance Criteria:**
- [ ] Frontend can call `check_ollama_status` and receive boolean
- [ ] Frontend receives `llm-chunk` events during streaming
- [ ] Frontend receives `llm-complete` event with full response
- [ ] Frontend receives `llm-error` event on failure

---

### Task 1.5: Basic Chat UI

**File: `src/components/chat/ChatInterface.tsx`**

```typescript
import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { MessageList } from './MessageList';
import { useConversationStore } from '@/stores/conversationStore';

interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
}

export function ChatInterface() {
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [streamingContent, setStreamingContent] = useState('');
  const { messages, addMessage, updateLastMessage } = useConversationStore();
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // Listen for streaming chunks
    const unlistenChunk = listen<string>('llm-chunk', (event) => {
      setStreamingContent(prev => prev + event.payload);
    });

    // Listen for completion
    const unlistenComplete = listen<string>('llm-complete', (event) => {
      setIsLoading(false);
      setStreamingContent('');
      addMessage({
        id: crypto.randomUUID(),
        role: 'assistant',
        content: event.payload,
        timestamp: new Date(),
      });
    });

    // Listen for errors
    const unlistenError = listen<string>('llm-error', (event) => {
      setIsLoading(false);
      setStreamingContent('');
      console.error('LLM Error:', event.payload);
    });

    return () => {
      unlistenChunk.then(f => f());
      unlistenComplete.then(f => f());
      unlistenError.then(f => f());
    };
  }, [addMessage]);

  // Auto-scroll to bottom
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, streamingContent]);

  const handleSubmit = async () => {
    if (!input.trim() || isLoading) return;

    const userMessage: Message = {
      id: crypto.randomUUID(),
      role: 'user',
      content: input.trim(),
      timestamp: new Date(),
    };

    addMessage(userMessage);
    setInput('');
    setIsLoading(true);
    setStreamingContent('');

    // Build message history for context
    const chatMessages = messages.concat(userMessage).map(m => ({
      role: m.role,
      content: m.content,
    }));

    try {
      await invoke('send_message', { messages: chatMessages });
    } catch (error) {
      console.error('Failed to send message:', error);
      setIsLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Messages area */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        <MessageList messages={messages} />

        {/* Streaming response */}
        {streamingContent && (
          <div className="p-4 rounded-lg bg-muted">
            <p className="text-sm text-muted-foreground mb-1">Assistant</p>
            <p className="whitespace-pre-wrap">{streamingContent}</p>
            <span className="inline-block w-2 h-4 bg-primary animate-pulse" />
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input area */}
      <div className="border-t p-4">
        <div className="flex gap-2">
          <Textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Describe your product idea..."
            className="flex-1 min-h-[80px] resize-none"
            disabled={isLoading}
          />
          <Button
            onClick={handleSubmit}
            disabled={isLoading || !input.trim()}
            className="self-end"
          >
            {isLoading ? 'Generating...' : 'Send'}
          </Button>
        </div>
      </div>
    </div>
  );
}
```

**File: `src/stores/conversationStore.ts`**

```typescript
import { create } from 'zustand';

interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
}

interface ConversationState {
  messages: Message[];
  addMessage: (message: Message) => void;
  updateLastMessage: (content: string) => void;
  clearMessages: () => void;
}

export const useConversationStore = create<ConversationState>((set) => ({
  messages: [],
  addMessage: (message) => set((state) => ({
    messages: [...state.messages, message]
  })),
  updateLastMessage: (content) => set((state) => ({
    messages: state.messages.map((m, i) =>
      i === state.messages.length - 1 ? { ...m, content } : m
    )
  })),
  clearMessages: () => set({ messages: [] }),
}));
```

**Acceptance Criteria:**
- [ ] User can type message and press Enter or click Send
- [ ] User message appears immediately in chat
- [ ] Streaming response displays character by character
- [ ] Completed response replaces streaming display
- [ ] Chat auto-scrolls to bottom on new messages

---

## Phase 2: Core Value - Implementation Plan Generation ⭐

This is the priority feature. Generate detailed, file-by-file implementation plans that AI coding agents can execute directly.

### Task 2.1: Implementation Plan Prompt Template

**File: `src-tauri/src/prompts/implementation_plan.txt`**

```text
You are SpecMaker, an expert software architect who creates detailed implementation plans optimized for AI coding agents like Claude Code and Cursor.

## Your Task
Generate a comprehensive implementation plan for the following product idea. The plan must be detailed enough that an AI coding agent can implement it without further clarification.

## Product Context
{product_description}

## Industry/Domain
{industry}

## Target Audience
{target_audience}

## Requirements Gathered
{requirements_summary}

## Output Format

Generate a Markdown document with the following structure:

# Implementation Plan: {project_name}

## Overview
[2-3 sentence summary of what will be built]

## Technology Stack
| Category | Technology | Version | Rationale |
|----------|------------|---------|-----------|
[Fill in recommended stack based on requirements]

## Project Structure
```
project-root/
├── [directory structure with comments explaining each folder/file]
```

## Implementation Phases

### Phase 1: [Title] (Core Foundation)
**Scope:** [What's included]
**Not in Scope:** [What's explicitly excluded - critical for AI agents]

#### Task 1.1: [Specific Task Title]
**Estimated Effort:** [XS/S/M/L/XL]

**Files to Create/Modify:**
| File Path | Purpose | Priority |
|-----------|---------|----------|
| `path/to/file.ts` | [Clear purpose] | Required |

**Prerequisites:**
- [ ] [What must be done first]

**Detailed Steps:**
1. [Atomic, specific instruction]
2. [Next instruction]
3. [Continue...]

**Code Template:**
```[language]
// Provide starter code or complete implementation
// Include all imports, types, and exports
```

**Acceptance Criteria:**
- [ ] [Specific, testable criterion]
- [ ] [Another criterion]

**Verification Commands:**
```bash
# Commands to verify this task is complete
```

**Common Pitfalls:**
- [What to avoid]

[Repeat for each task in phase]

### Phase 2: [Title]
[Same structure...]

## Data Models

### Entity: [Name]
```typescript
interface EntityName {
  id: string;
  // ... all fields with types and descriptions
}
```

**Database Schema:**
```sql
CREATE TABLE entity_name (
  -- Complete schema
);
```

## API Specifications (if applicable)

### Endpoint: [Name]
- **Method:** POST/GET/etc.
- **Path:** `/api/endpoint`
- **Request Body:**
```json
{
  "field": "type and description"
}
```
- **Response:**
```json
{
  "field": "type and description"
}
```
- **Error Codes:** [List with meanings]

## Testing Strategy

### Unit Tests
| Component | Test File | Coverage Target |
|-----------|-----------|-----------------|
| [Component] | `tests/component.test.ts` | 80% |

### Integration Tests
[Describe integration test approach]

### E2E Tests
| Flow | Test File | Critical Path |
|------|-----------|---------------|
| [User flow] | `e2e/flow.test.ts` | Yes/No |

## Deployment Checklist
- [ ] [Deployment step 1]
- [ ] [Deployment step 2]

## Security Considerations
- [ ] [Security requirement 1]
- [ ] [Security requirement 2]

---

## Critical Instructions for AI Agents

1. **Follow phases sequentially** - Each phase builds on the previous
2. **Check prerequisites** - Verify dependencies before starting a task
3. **Run verification commands** - Confirm each task before proceeding
4. **Respect "Not in Scope"** - Do not implement features marked as out of scope
5. **Use exact file paths** - Create files exactly where specified
6. **Match code templates** - Use provided code as starting point, don't deviate significantly
```

**Acceptance Criteria:**
- [ ] Prompt template includes all sections
- [ ] Placeholders clearly marked with `{variable_name}`
- [ ] AI agent instructions included at the end

---

### Task 2.2: Plan Generation Service

**File: `src-tauri/src/services/plan_generator.rs`**

```rust
use crate::services::ollama::{OllamaService, ChatMessage};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanContext {
    pub project_name: String,
    pub product_description: String,
    pub industry: Option<String>,
    pub target_audience: Option<String>,
    pub requirements_summary: String,
}

pub struct PlanGenerator<'a> {
    ollama: &'a OllamaService,
    templates: Handlebars<'a>,
}

impl<'a> PlanGenerator<'a> {
    pub fn new(ollama: &'a OllamaService) -> Self {
        let mut templates = Handlebars::new();
        templates
            .register_template_string(
                "implementation_plan",
                include_str!("../prompts/implementation_plan.txt"),
            )
            .expect("Failed to register implementation plan template");

        Self { ollama, templates }
    }

    pub async fn generate_implementation_plan(
        &self,
        context: PlanContext,
    ) -> Result<String, String> {
        // Render the prompt with context
        let prompt = self
            .templates
            .render("implementation_plan", &context)
            .map_err(|e| format!("Template error: {}", e))?;

        // Build messages
        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are SpecMaker, generating implementation plans for AI coding agents.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ];

        // Generate with higher token limit for comprehensive plans
        self.ollama.generate(messages).await
    }
}
```

**Acceptance Criteria:**
- [ ] Template renders correctly with all variables
- [ ] Generated plan follows the specified structure
- [ ] Plan includes file-by-file breakdowns
- [ ] Plan includes acceptance criteria for each task

---

### Task 2.3: Implementation Plan UI

**File: `src/components/artifacts/ImplementationPlanViewer.tsx`**

```typescript
import { useState } from 'react';
import ReactMarkdown from 'react-markdown';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { Button } from '@/components/ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Copy, Download, Check } from 'lucide-react';

interface ImplementationPlanViewerProps {
  content: string;
  title: string;
  onExport: (format: 'md' | 'html' | 'pdf') => void;
}

export function ImplementationPlanViewer({
  content,
  title,
  onExport
}: ImplementationPlanViewerProps) {
  const [copied, setCopied] = useState(false);
  const [activeTab, setActiveTab] = useState<'preview' | 'source'>('preview');

  const handleCopy = async () => {
    await navigator.clipboard.writeText(content);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b">
        <h2 className="text-lg font-semibold">{title}</h2>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={handleCopy}>
            {copied ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
            {copied ? 'Copied!' : 'Copy'}
          </Button>
          <Button variant="outline" size="sm" onClick={() => onExport('md')}>
            <Download className="w-4 h-4 mr-1" />
            Export
          </Button>
        </div>
      </div>

      {/* Content */}
      <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as any)} className="flex-1">
        <TabsList className="mx-4 mt-2">
          <TabsTrigger value="preview">Preview</TabsTrigger>
          <TabsTrigger value="source">Source</TabsTrigger>
        </TabsList>

        <TabsContent value="preview" className="flex-1 overflow-y-auto p-4">
          <article className="prose prose-slate dark:prose-invert max-w-none">
            <ReactMarkdown
              components={{
                code({ node, inline, className, children, ...props }) {
                  const match = /language-(\w+)/.exec(className || '');
                  return !inline && match ? (
                    <SyntaxHighlighter
                      style={oneDark}
                      language={match[1]}
                      PreTag="div"
                      {...props}
                    >
                      {String(children).replace(/\n$/, '')}
                    </SyntaxHighlighter>
                  ) : (
                    <code className={className} {...props}>
                      {children}
                    </code>
                  );
                },
                // Custom checkbox rendering for acceptance criteria
                input({ type, checked, ...props }) {
                  if (type === 'checkbox') {
                    return (
                      <input
                        type="checkbox"
                        checked={checked}
                        readOnly
                        className="mr-2 accent-primary"
                        {...props}
                      />
                    );
                  }
                  return <input type={type} {...props} />;
                },
              }}
            >
              {content}
            </ReactMarkdown>
          </article>
        </TabsContent>

        <TabsContent value="source" className="flex-1 overflow-y-auto">
          <SyntaxHighlighter
            language="markdown"
            style={oneDark}
            customStyle={{ margin: 0, height: '100%' }}
          >
            {content}
          </SyntaxHighlighter>
        </TabsContent>
      </Tabs>
    </div>
  );
}
```

**Acceptance Criteria:**
- [ ] Markdown renders with proper formatting
- [ ] Code blocks have syntax highlighting
- [ ] Checkboxes render for acceptance criteria
- [ ] Copy button copies raw markdown
- [ ] Tab switching between preview and source works
- [ ] Export button triggers callback

---

### Task 2.4: Quick Generation Flow

For rapid iteration, allow users to go directly from product idea to implementation plan.

**File: `src/components/QuickGenerateFlow.tsx`**

```typescript
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Loader2, Sparkles } from 'lucide-react';
import { ImplementationPlanViewer } from './artifacts/ImplementationPlanViewer';

export function QuickGenerateFlow() {
  const [step, setStep] = useState<'input' | 'generating' | 'result'>('input');
  const [formData, setFormData] = useState({
    projectName: '',
    productDescription: '',
    industry: '',
    targetAudience: '',
  });
  const [generatedPlan, setGeneratedPlan] = useState('');
  const [error, setError] = useState('');

  const handleGenerate = async () => {
    if (!formData.projectName || !formData.productDescription) {
      setError('Project name and description are required');
      return;
    }

    setStep('generating');
    setError('');

    try {
      const plan = await invoke<string>('generate_implementation_plan', {
        context: {
          project_name: formData.projectName,
          product_description: formData.productDescription,
          industry: formData.industry || null,
          target_audience: formData.targetAudience || null,
          requirements_summary: formData.productDescription, // Use description as initial requirements
        },
      });
      setGeneratedPlan(plan);
      setStep('result');
    } catch (e) {
      setError(String(e));
      setStep('input');
    }
  };

  const handleExport = async (format: 'md' | 'html' | 'pdf') => {
    await invoke('export_artifact', {
      content: generatedPlan,
      format,
      filename: `${formData.projectName.toLowerCase().replace(/\s+/g, '-')}-implementation-plan`,
    });
  };

  if (step === 'generating') {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <Loader2 className="w-12 h-12 animate-spin text-primary" />
        <p className="text-lg">Generating implementation plan...</p>
        <p className="text-sm text-muted-foreground">This may take 1-2 minutes</p>
      </div>
    );
  }

  if (step === 'result') {
    return (
      <div className="h-full flex flex-col">
        <div className="p-4 border-b flex justify-between items-center">
          <Button variant="outline" onClick={() => setStep('input')}>
            ← New Plan
          </Button>
        </div>
        <div className="flex-1">
          <ImplementationPlanViewer
            content={generatedPlan}
            title={`${formData.projectName} - Implementation Plan`}
            onExport={handleExport}
          />
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-2xl mx-auto p-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Sparkles className="w-5 h-5" />
            Quick Implementation Plan
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {error && (
            <div className="p-3 bg-destructive/10 text-destructive rounded-md">
              {error}
            </div>
          )}

          <div className="space-y-2">
            <Label htmlFor="projectName">Project Name *</Label>
            <Input
              id="projectName"
              value={formData.projectName}
              onChange={(e) => setFormData({ ...formData, projectName: e.target.value })}
              placeholder="e.g., TaskFlow Pro"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="productDescription">Product Description *</Label>
            <Textarea
              id="productDescription"
              value={formData.productDescription}
              onChange={(e) => setFormData({ ...formData, productDescription: e.target.value })}
              placeholder="Describe your product idea in detail. What problem does it solve? Who is it for? What are the key features?"
              className="min-h-[150px]"
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="industry">Industry (optional)</Label>
              <Input
                id="industry"
                value={formData.industry}
                onChange={(e) => setFormData({ ...formData, industry: e.target.value })}
                placeholder="e.g., FinTech, Healthcare"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="targetAudience">Target Audience (optional)</Label>
              <Input
                id="targetAudience"
                value={formData.targetAudience}
                onChange={(e) => setFormData({ ...formData, targetAudience: e.target.value })}
                placeholder="e.g., Small business owners"
              />
            </div>
          </div>

          <Button onClick={handleGenerate} className="w-full" size="lg">
            <Sparkles className="w-4 h-4 mr-2" />
            Generate Implementation Plan
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
```

**Acceptance Criteria:**
- [ ] Form validates required fields
- [ ] Loading state shows while generating
- [ ] Generated plan displays in viewer
- [ ] User can create new plan from result screen
- [ ] Export works from result screen

---

## Phase 3: Enhanced Consultation & PRD

### Task 3.1: Multi-Phase Consultation Prompts

**File: `src-tauri/src/prompts/consultation_phases.rs`**

```rust
pub const INITIAL_ANALYSIS_PROMPT: &str = r#"
You are SpecMaker, an expert product consultant. Analyze the following product idea and generate 5 clarifying questions to better understand the requirements.

Product Idea:
{product_description}

Generate questions that cover:
1. **User Pain Points** - What specific problems are we solving?
2. **Success Metrics** - How will we measure success?
3. **Technical Constraints** - Any platform, technology, or integration requirements?
4. **Scope Boundaries** - What should NOT be included in v1?
5. **Competitive Landscape** - What existing solutions are users comparing against?

Format your response as:
## Analysis
[2-3 sentences summarizing your understanding]

## Clarifying Questions
1. [Question about pain points]
2. [Question about success metrics]
3. [Question about constraints]
4. [Question about scope]
5. [Question about competition/differentiation]
"#;

pub const CONSULTATION_PROMPT: &str = r#"
Based on the conversation so far, continue gathering requirements. Focus on areas not yet covered.

Previous context:
{conversation_summary}

Latest user response:
{user_response}

Either:
1. Ask 2-3 more clarifying questions if important areas remain unclear
2. Summarize your complete understanding if you have enough information

Format:
## Understanding Update
[What you now understand better]

## Questions (if needed)
1. [Question]
2. [Question]

OR

## Requirements Summary
[Comprehensive summary ready for PRD generation]
"#;

pub const PRD_GENERATION_PROMPT: &str = r#"
Generate a comprehensive Product Requirements Document based on the following gathered requirements.

Project: {project_name}
Industry: {industry}
Target Audience: {target_audience}

Requirements Summary:
{requirements_summary}

Generate a PRD following this structure:

# Product Requirements Document: {project_name}

## Executive Summary
[2-3 paragraphs: what, why, for whom]

## Problem Statement
### Current Situation
[What exists today]

### Pain Points
[Specific problems users face - bulleted list]

### Opportunity
[Why now is the right time]

## Goals & Objectives

### Business Goals
1. [Measurable goal]
2. [Measurable goal]

### User Goals
1. [What users want to achieve]
2. [What users want to achieve]

### Non-Goals (Explicitly Out of Scope)
1. [What we are NOT building]
2. [Future considerations, not v1]

## User Personas

### Primary Persona: [Name]
- **Role:** [Job title/description]
- **Background:** [Context]
- **Goals:** [What they want]
- **Pain Points:** [Current frustrations]
- **Success Criteria:** [How they'll judge the product]

[Repeat for 2-3 personas]

## Functional Requirements

### Core Features (Must Have)
| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| FR-01 | [Requirement] | P0 | [Criteria] |

### Secondary Features (Should Have)
| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| FR-10 | [Requirement] | P1 | [Criteria] |

### Nice-to-Have Features
| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| FR-20 | [Requirement] | P2 | [Criteria] |

## Non-Functional Requirements

### Performance
- [Specific metric and target]

### Security
- [Security requirement]

### Scalability
- [Scalability requirement]

### Accessibility
- [Accessibility requirement]

## User Stories

[Group by feature area]

### [Feature Area 1]

**US-001: [Story Title]**
- **As a** [persona]
- **I want** [capability]
- **So that** [benefit]
- **Acceptance Criteria:**
  - [ ] [Criterion]
  - [ ] [Criterion]

[Continue with more stories]

## Success Metrics

| Metric | Baseline | Target | Measurement Method |
|--------|----------|--------|-------------------|
| [Metric] | [Current] | [Goal] | [How to measure] |

## Implementation Phases

### Phase 1: MVP ([X] weeks)
- [Feature 1]
- [Feature 2]
- **Success Criteria:** [How to know phase is complete]

### Phase 2: Enhancement ([X] weeks)
- [Feature 3]
- [Feature 4]

[Continue phases]

## Open Questions
- [Unresolved item requiring decision]

## Appendix
### Glossary
### References
"#;
```

**Acceptance Criteria:**
- [ ] Initial analysis generates exactly 5 questions
- [ ] Consultation adapts based on previous responses
- [ ] PRD follows complete structure
- [ ] All sections populated with relevant content

---

### Task 3.2: Tech Spec Generation

**File: `src-tauri/src/prompts/tech_spec.txt`**

```text
Generate a Technical Specification document based on the PRD.

PRD Summary:
{prd_summary}

Technology Preferences (if any):
{tech_preferences}

Generate a comprehensive tech spec:

# Technical Specification: {project_name}

## Document Info
- **Related PRD:** {prd_reference}
- **Version:** 1.0
- **Status:** Draft

## Architecture Overview

### System Context Diagram
```mermaid
C4Context
    [Diagram showing system boundaries and external actors]
```

### High-Level Architecture
```mermaid
flowchart TB
    [Architecture diagram]
```

### Architecture Decision Records

| Decision | Options Considered | Choice | Rationale |
|----------|-------------------|--------|-----------|
| [Decision] | A, B, C | B | [Why] |

## Technology Stack

### Frontend
| Category | Technology | Version | Justification |
|----------|------------|---------|---------------|
| Framework | | | |
| UI Library | | | |
| State Mgmt | | | |

### Backend
| Category | Technology | Version | Justification |
|----------|------------|---------|---------------|
| Runtime | | | |
| Framework | | | |
| Database | | | |

### Infrastructure
| Category | Technology | Justification |
|----------|------------|---------------|
| Hosting | | |
| CI/CD | | |

## Data Models

### Entity Relationship Diagram
```mermaid
erDiagram
    [ER diagram]
```

### Schema Definitions

#### Table: [name]
```sql
CREATE TABLE [name] (
    [complete schema]
);
```

## API Design

### REST Endpoints

#### [Resource Name]

**Create [Resource]**
- **Method:** POST
- **Path:** `/api/[resources]`
- **Auth:** Required
- **Request:**
```json
{
    [request body]
}
```
- **Response (201):**
```json
{
    [response body]
}
```
- **Errors:**
  - 400: [reason]
  - 401: [reason]

[Continue for all endpoints]

## Security Architecture

### Authentication
[Describe auth approach]

### Authorization
[Describe authz approach]

### Data Protection
[Describe encryption, PII handling]

### Security Checklist
- [ ] Input validation on all endpoints
- [ ] Parameterized queries (SQL injection prevention)
- [ ] Output encoding (XSS prevention)
- [ ] HTTPS enforced
- [ ] Secrets in environment variables
- [ ] Rate limiting implemented

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Page Load | <2s | Lighthouse |
| API Response | <500ms | Server logs |
| Database Query | <100ms | Query profiling |

## Testing Strategy

### Unit Tests
- **Framework:** [Framework]
- **Coverage Target:** 80%
- **Focus:** Pure functions, business logic

### Integration Tests
- **Framework:** [Framework]
- **Focus:** API endpoints, database operations

### E2E Tests
- **Framework:** [Framework]
- **Critical Paths:**
  - [Path 1]
  - [Path 2]

## Deployment

### Environment Configuration
| Variable | Development | Production |
|----------|-------------|------------|
| [VAR] | [value] | [value] |

### Deployment Pipeline
1. [Step 1]
2. [Step 2]

### Rollback Procedure
[How to revert]

## Monitoring & Observability

### Logging
- **Format:** JSON structured
- **Levels:** ERROR, WARN, INFO, DEBUG

### Metrics
| Metric | Alert Threshold |
|--------|-----------------|
| [Metric] | [Threshold] |

### Alerting
[Alert configuration]
```

**Acceptance Criteria:**
- [ ] Tech spec includes Mermaid diagrams
- [ ] All architecture decisions documented with rationale
- [ ] Complete API specifications
- [ ] Security checklist included
- [ ] Performance targets defined

---

## Phase 4: Polish & Production

### Task 4.1: Export Service

**File: `src-tauri/src/services/export.rs`**

```rust
use std::path::PathBuf;
use std::fs;
use tauri::api::dialog::FileDialogBuilder;

pub enum ExportFormat {
    Markdown,
    Html,
    Pdf,
}

pub struct ExportService;

impl ExportService {
    /// Export artifact to file
    pub async fn export(
        content: &str,
        format: ExportFormat,
        suggested_filename: &str,
    ) -> Result<PathBuf, String> {
        let extension = match format {
            ExportFormat::Markdown => "md",
            ExportFormat::Html => "html",
            ExportFormat::Pdf => "pdf",
        };

        let filename = format!("{}.{}", suggested_filename, extension);

        // Show save dialog
        let path = FileDialogBuilder::new()
            .set_file_name(&filename)
            .save_file()
            .ok_or("Export cancelled")?;

        let output = match format {
            ExportFormat::Markdown => content.to_string(),
            ExportFormat::Html => Self::markdown_to_html(content),
            ExportFormat::Pdf => {
                // For PDF, first convert to HTML then use system print
                let html = Self::markdown_to_html(content);
                Self::html_to_pdf(&html, &path).await?;
                return Ok(path);
            }
        };

        fs::write(&path, output).map_err(|e| e.to_string())?;
        Ok(path)
    }

    fn markdown_to_html(markdown: &str) -> String {
        use pulldown_cmark::{Parser, Options, html};

        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_STRIKETHROUGH);

        let parser = Parser::new_ext(markdown, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        // Wrap in styled HTML document
        format!(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            line-height: 1.6;
            max-width: 900px;
            margin: 0 auto;
            padding: 2rem;
            color: #333;
        }}
        h1, h2, h3 {{ color: #1a1a1a; }}
        code {{
            background: #f4f4f4;
            padding: 0.2em 0.4em;
            border-radius: 3px;
            font-size: 0.9em;
        }}
        pre {{
            background: #282c34;
            color: #abb2bf;
            padding: 1rem;
            border-radius: 5px;
            overflow-x: auto;
        }}
        pre code {{
            background: none;
            padding: 0;
        }}
        table {{
            border-collapse: collapse;
            width: 100%;
            margin: 1rem 0;
        }}
        th, td {{
            border: 1px solid #ddd;
            padding: 0.75rem;
            text-align: left;
        }}
        th {{ background: #f8f9fa; }}
        blockquote {{
            border-left: 4px solid #ddd;
            margin: 0;
            padding-left: 1rem;
            color: #666;
        }}
        input[type="checkbox"] {{
            margin-right: 0.5rem;
        }}
    </style>
</head>
<body>
{html_output}
</body>
</html>
"#)
    }

    async fn html_to_pdf(html: &str, output_path: &PathBuf) -> Result<(), String> {
        // Use system's print-to-PDF capability or headless browser
        // For macOS, we can use wkhtmltopdf if available, or fall back to HTML

        // Simple approach: save as HTML and let user print to PDF
        // For production, integrate headless Chrome or wkhtmltopdf

        let html_path = output_path.with_extension("html");
        fs::write(&html_path, html).map_err(|e| e.to_string())?;

        // Open in default browser for manual PDF export
        // In production, use headless rendering
        open::that(&html_path).map_err(|e| e.to_string())?;

        Ok(())
    }
}
```

**Acceptance Criteria:**
- [ ] Markdown export preserves all formatting
- [ ] HTML export styled professionally
- [ ] PDF export opens printable HTML (v1) or generates PDF directly (v2)
- [ ] Save dialog shows with suggested filename

---

### Task 4.2: Settings Panel

**File: `src/components/settings/SettingsPanel.tsx`**

```typescript
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Slider } from '@/components/ui/slider';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { CheckCircle, XCircle, RefreshCw } from 'lucide-react';

interface OllamaConfig {
  model: string;
  temperature: number;
  max_tokens: number | null;
}

const RECOMMENDED_MODELS = [
  { id: 'llama3.1:70b', name: 'Llama 3.1 70B', description: 'Best quality, slower' },
  { id: 'llama3.1:8b', name: 'Llama 3.1 8B', description: 'Fast, good quality' },
  { id: 'mixtral:8x7b', name: 'Mixtral 8x7B', description: 'Balanced performance' },
  { id: 'codellama:34b', name: 'CodeLlama 34B', description: 'Code-focused' },
];

export function SettingsPanel() {
  const [config, setConfig] = useState<OllamaConfig>({
    model: 'llama3.1:8b',
    temperature: 0.7,
    max_tokens: 4096,
  });
  const [ollamaStatus, setOllamaStatus] = useState<'checking' | 'connected' | 'disconnected'>('checking');
  const [availableModels, setAvailableModels] = useState<string[]>([]);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    checkOllamaStatus();
  }, []);

  const checkOllamaStatus = async () => {
    setOllamaStatus('checking');
    try {
      const connected = await invoke<boolean>('check_ollama_status');
      setOllamaStatus(connected ? 'connected' : 'disconnected');

      if (connected) {
        const models = await invoke<string[]>('list_ollama_models');
        setAvailableModels(models);
      }
    } catch {
      setOllamaStatus('disconnected');
    }
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await invoke('update_ollama_config', { config });
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="max-w-2xl mx-auto p-6 space-y-6">
      {/* Ollama Status */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            Ollama Connection
            <Button variant="ghost" size="sm" onClick={checkOllamaStatus}>
              <RefreshCw className="w-4 h-4" />
            </Button>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            {ollamaStatus === 'checking' && (
              <Badge variant="outline">Checking...</Badge>
            )}
            {ollamaStatus === 'connected' && (
              <>
                <CheckCircle className="w-5 h-5 text-green-500" />
                <span className="text-green-700">Connected to Ollama</span>
              </>
            )}
            {ollamaStatus === 'disconnected' && (
              <>
                <XCircle className="w-5 h-5 text-red-500" />
                <span className="text-red-700">Ollama not running</span>
              </>
            )}
          </div>
          {ollamaStatus === 'disconnected' && (
            <p className="mt-2 text-sm text-muted-foreground">
              Start Ollama with: <code className="bg-muted px-1 rounded">ollama serve</code>
            </p>
          )}
        </CardContent>
      </Card>

      {/* Model Selection */}
      <Card>
        <CardHeader>
          <CardTitle>Model Configuration</CardTitle>
          <CardDescription>Choose the AI model for generation</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label>Model</Label>
            <Select
              value={config.model}
              onValueChange={(value) => setConfig({ ...config, model: value })}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {RECOMMENDED_MODELS.map((model) => (
                  <SelectItem key={model.id} value={model.id}>
                    <div>
                      <span className="font-medium">{model.name}</span>
                      <span className="text-muted-foreground ml-2 text-sm">
                        {model.description}
                      </span>
                      {availableModels.includes(model.id) && (
                        <Badge variant="outline" className="ml-2">Installed</Badge>
                      )}
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <div className="flex justify-between">
              <Label>Temperature</Label>
              <span className="text-sm text-muted-foreground">{config.temperature}</span>
            </div>
            <Slider
              value={[config.temperature]}
              onValueChange={([value]) => setConfig({ ...config, temperature: value })}
              min={0}
              max={1}
              step={0.1}
            />
            <p className="text-xs text-muted-foreground">
              Lower = more focused, Higher = more creative
            </p>
          </div>

          <div className="space-y-2">
            <Label>Max Tokens</Label>
            <Input
              type="number"
              value={config.max_tokens || ''}
              onChange={(e) => setConfig({
                ...config,
                max_tokens: e.target.value ? parseInt(e.target.value) : null
              })}
              placeholder="4096"
            />
          </div>

          <Button onClick={handleSave} disabled={saving}>
            {saving ? 'Saving...' : 'Save Configuration'}
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
```

**Acceptance Criteria:**
- [ ] Ollama connection status updates correctly
- [ ] Model dropdown shows installed status
- [ ] Temperature slider works smoothly
- [ ] Settings persist after save

---

## Verification Strategy

### End-to-End Test Flow

1. **App Launch**
   - [ ] Window opens with correct title
   - [ ] Ollama status checked and displayed

2. **Quick Generate Flow**
   - [ ] Fill form with test product idea
   - [ ] Click generate
   - [ ] Plan appears in viewer within 2 minutes
   - [ ] Plan contains all required sections
   - [ ] Copy button copies markdown
   - [ ] Export creates file

3. **Generated Plan Quality Check**
   - [ ] Includes project structure diagram
   - [ ] Includes file-by-file task breakdowns
   - [ ] Each task has acceptance criteria
   - [ ] Includes verification commands
   - [ ] Includes "Not in Scope" sections
   - [ ] Plan is detailed enough for Claude Code to implement

4. **Settings**
   - [ ] Can change model
   - [ ] Settings persist after restart

---

## Success Metrics

### Phase 1 Complete When:
- [ ] App launches and connects to Ollama
- [ ] Basic chat works with streaming
- [ ] Messages persist in SQLite

### Phase 2 Complete When: ⭐
- [ ] Quick generate produces detailed implementation plans
- [ ] Plans include file-by-file breakdowns
- [ ] Plans include acceptance criteria per task
- [ ] Claude Code can implement a plan without clarification (test with simple project)

### Phase 3 Complete When:
- [ ] Multi-turn consultation refines requirements
- [ ] Full PRD generated with all sections
- [ ] Tech spec includes architecture diagrams

### Phase 4 Complete When:
- [ ] Export to MD/HTML/PDF works
- [ ] Settings panel functional
- [ ] App feels polished and production-ready

---

## Implementation Notes for Claude Code

1. **Start with Phase 1 sequentially** - Each task builds on previous
2. **Run verification after each task** - Don't proceed if tests fail
3. **Use exact file paths specified** - Directory structure matters
4. **Copy code templates exactly** - Then modify as needed
5. **Ask for clarification** - If any specification is ambiguous
6. **Test Ollama integration early** - It's the foundation of everything

---

## Appendix: Dependencies

### Rust (`Cargo.toml`)
```toml
[package]
name = "spec-maker"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2.0", features = [] }

[dependencies]
tauri = { version = "2.0", features = ["shell-open", "dialog-all"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
futures-util = "0.3"
rusqlite = { version = "0.31", features = ["bundled"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
handlebars = "5.0"
pulldown-cmark = "0.10"
open = "5.0"
```

### TypeScript (`package.json`)
```json
{
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "react": "^18.3.0",
    "react-dom": "^18.3.0",
    "react-markdown": "^9.0.0",
    "react-syntax-highlighter": "^15.5.0",
    "zustand": "^4.5.0",
    "lucide-react": "^0.400.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.3.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@types/react": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0",
    "tailwindcss": "^3.4.0",
    "typescript": "^5.4.0",
    "vite": "^5.3.0"
  }
}
```
