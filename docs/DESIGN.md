# Astryx Design System Analysis & pseudo-harness WebApp Design Specification

## 1. Executive Summary & Synthesis

This document distills the design patterns, architectural philosophy, and token structures of **Astryx** (`meta/astryx`), combined with the cyberpunk-industrial and tactile UI aesthetic references (`refs/`), into a concrete Design System and UI specification for the **`pseudo-harness` v3.1 Web Application Workbench**.

---

## 2. Astryx Design System Principles

1. **Agent-Ready & Human-Centric Co-Design:** APIs, documentation, and tooling (CLI/AGENTS.md) are designed so humans and AI agents build using identical paradigms.
2. **Open Internals:** Components are composed at any level without being locked behind closed abstractions; swizzling exposes full source ownership.
3. **No Styling Lock-In:** Styled via CSS custom properties and StyleX without framework lock-in, permitting direct overrides and clean integration.
4. **Cohesive Themes & Foundations:** Built-in modular themes (neutral, butter, matcha, gothic, y2k, stone) anchored to typography, spacing, and accessibility invariants.

---

## 3. Visual & Tactile References Analysis

From the reference imagery (`refs/`):
- **Technical Telemetry / Industrial Blueprint:** Monospaced data readouts, precision grid lines, bracketed metadata annotations (`[DELIVERY SYSTEMS]`), and high-contrast dark backdrops (`#121317` to `#1a1b26`).
- **Tactile Squircles & Neon Accents:** Organic rounded cards and pill buttons contrasted with sharp technical borders, glowing neon emeralds (`#73daca`), cyan highlights (`#7aa2f7`), and warning ambers (`#e0af68`).

---

## 4. Typography & Spacing Token System

### A. Typography Hierarchy
- **Display / Monospace Code:** `JetBrains Mono`, `Fira Code`, or `SF Mono` (Monospaced, tabular numbers, strict alignment for pIR and AST streams).
- **UI & Body:** Inter / system sans-serif for clean readability across 4-region workbench panels.
- **Scale:**
  - `text-xs` (11px): Telemetry badges, footer hints, token counts.
  - `text-sm` (13px): Editor buffers, symbol trees, AI stream logs.
  - `text-base` (14px): Prompt input, modal headers.
  - `text-lg` (16px): Workbench main title.

### B. Spacing & Radius Tokens
- **Base Grid Unit:** `4px` (`--space-1: 4px`, `--space-2: 8px`, `--space-3: 12px`, `--space-4: 16px`).
- **Border Radius:**
  - Sharp technical containers: `4px` (`--radius-sm`).
  - Tactile floating popovers and drawers: `8px` (`--radius-md`).
  - Organic pill buttons and badges: `16px` (`--radius-pill`).

---

## 5. pseudo-harness v3.1 WebApp Layout & UI Architecture

The workbench implements a 4-region layout served by the Axum backend (`src/server.rs`) and rendered by the Svelte SPA (`static/index.html`):

1. **Top Status Bar (1 row):** Live telemetry (`● ENGINE: 3.2B TERNARY`, VRAM usage, Lamport tick, and color-coded token budget meter).
2. **Region 1: File & AST Tree Explorer (Left Sidebar, ~22%):** Workspace file browser and AST symbol nodes.
3. **Region 2: Active Editor (Top-Right Canvas, ~60% height):** `pseudo-lang` authoring buffer with syntax highlighting and sandbox eval triggers.
4. **Region 3: AI Assistant Stream (Bottom-Right Panel, ~40% height):** Live streaming output from `psudo-engine` showing `<talk>` reasoning and `<pir>` generated blocks.
5. **Region 4: Operator Prompt & `@` Completion Popover (Bottom bar):** Prompt input equipped with real-time fuzzy auto-complete for symbols and files (`@net::client::fetch`).
6. **Modal Drawers:** Agenda Drawer (`Ctrl+B`), Repo Map Drawer (`Ctrl+M`), and VM Diagnostics (`Ctrl+D`).
