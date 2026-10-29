<script>
  import { onMount } from 'svelte';
  import {
    Cpu, HardDrive, Activity, Gauge, FolderTree, FileCode, CheckSquare,
    Box, FileText, BookOpen, Code, Play, Bot, Zap, Terminal, AtSign,
    ListTodo, GitMerge, ShieldAlert, Send, X, Server, Command, CheckCircle2,
    Sparkles, ShieldCheck
  } from 'lucide-svelte';

  // State
  let model = "3.2B TERNARY";
  let vram = "1.4 / 6.0 GB";
  let tps = 42;
  let lamportTick = 142;
  let tokenCount = 340;
  const tokenCeiling = 1200;

  let activeFile = "src/main.pseudo";
  let editorContent = `// pseudo-lang interactive workbench
f fetch_user (url:s) -> :d [
    v t 5000
    v res (@x "http" "get" {url: url, timeout_ms: t})
    ? (== res.status 200) [
        r {"ok": true, "body": res.body}
    ] [
        r {"ok": false, "err": "network_failure"}
    ]
]

f main () [
    v user (fetch_user "https://api.example.com/user")
    @print user
    r 0
]`;

  let streamLogs = [
    { type: "talk", text: "> psudo-engine ready in VRAM.\nMe listen for prompt or @ references.\nMe emit tiny pIR opcodes." }
  ];

  let promptInput = "";
  let files = [
    "src/main.pseudo", "src/net/client.pseudo", ".pseudo/main.pir",
    ".pseudo/todo.json", "Cargo.toml", "README.md", "SPEC.md"
  ];
  let selectedFileIdx = 0;

  // Autocomplete state
  let showCompletion = false;
  let candidates = [];
  let selectedCandidateIdx = 0;

  // Drawers
  let showAgenda = false;
  let showRepoMap = false;
  let showDiagnostics = false;
  let agendaTasks = [];
  let repoSymbols = [];

  // Token percentage
  $: tokenPct = Math.min(100, Math.round((tokenCount / tokenCeiling) * 100));
  $: tokenBarClass = tokenPct > 90 ? "token-bar-fill crimson" : tokenPct > 65 ? "token-bar-fill amber" : "token-bar-fill";

  onMount(async () => {
    syncStatus();
    const interval = setInterval(syncStatus, 5000);
    window.addEventListener('keydown', handleGlobalKeydown);
    return () => {
      clearInterval(interval);
      window.removeEventListener('keydown', handleGlobalKeydown);
    };
  });

  async function syncStatus() {
    try {
      const res = await fetch('/api/status');
      if (res.ok) {
        const data = await res.json();
        model = data.model;
        vram = data.vram;
        tps = data.tokens_per_second;
        tokenCount = data.token_count;
        lamportTick = data.lamport_tick;
      }
    } catch (e) {
      console.warn("Backend sync:", e);
    }
  }

  async function handlePromptInput(e) {
    const val = e.target.value;
    const cursorPos = e.target.selectionStart;
    const textBeforeCursor = val.slice(0, cursorPos);
    const atIdx = textBeforeCursor.lastIndexOf('@');

    if (atIdx !== -1) {
      const query = textBeforeCursor.slice(atIdx + 1);
      if (!query.includes(' ')) {
        try {
          const res = await fetch(`/api/symbols?q=${encodeURIComponent(query)}`);
          if (res.ok) {
            candidates = await res.json();
            showCompletion = candidates.length > 0;
            selectedCandidateIdx = 0;
            return;
          }
        } catch (err) {
          console.error(err);
        }
      }
    }
    showCompletion = false;
  }

  function handlePromptKeydown(e) {
    if (showCompletion) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        selectedCandidateIdx = (selectedCandidateIdx + 1) % candidates.length;
        return;
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        selectedCandidateIdx = (selectedCandidateIdx - 1 + candidates.length) % candidates.length;
        return;
      } else if (e.key === 'Enter' || e.key === 'Tab') {
        e.preventDefault();
        if (candidates[selectedCandidateIdx]) {
          selectCandidate(candidates[selectedCandidateIdx]);
        }
        return;
      } else if (e.key === 'Escape') {
        showCompletion = false;
        return;
      }
    }

    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      submitPrompt();
    }
  }

  function selectCandidate(item) {
    const atIdx = promptInput.lastIndexOf('@');
    if (atIdx !== -1) {
      promptInput = promptInput.slice(0, atIdx) + '@' + item.name + ' ';
    }
    showCompletion = false;
  }

  async function submitPrompt() {
    const text = promptInput.trim();
    if (!text) return;

    streamLogs = [...streamLogs, { type: 'user', text: text }];
    promptInput = '';
    showCompletion = false;

    try {
      const res = await fetch('/api/prompt', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ prompt: text })
      });
      const data = await res.json();
      if (data.ok) {
        streamLogs = [...streamLogs, { type: 'talk', text: data.reply }];
        tokenCount = data.token_count;
      } else {
        streamLogs = [...streamLogs, { type: 'error', text: `[ERROR] ${data.error}` }];
      }
    } catch (err) {
      streamLogs = [...streamLogs, { type: 'error', text: '[NETWORK ERROR] Could not reach harness server' }];
    }
  }

  async function runSandbox() {
    streamLogs = [...streamLogs, { type: 'warn', text: '[SANDBOX EVAL] Validating pIR AST in hermetic sandbox...' }];
    try {
      const res = await fetch('/api/eval', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ code: editorContent })
      });
      const data = await res.json();
      if (data.ok) {
        streamLogs = [...streamLogs, { type: 'success', text: '[SANDBOX OK] In-memory verification passed (0 traps).' }];
      } else {
        streamLogs = [...streamLogs, { type: 'error', text: `[SANDBOX TRAP] ${data.error}` }];
      }
    } catch (err) {
      streamLogs = [...streamLogs, { type: 'error', text: '[SANDBOX FAILED] Execution service error' }];
    }
  }

  function loadFile(file, idx) {
    activeFile = file;
    selectedFileIdx = idx;
  }

  async function toggleDrawer(name) {
    if (name === 'agenda') {
      showAgenda = !showAgenda;
      if (showAgenda) {
        const res = await fetch('/api/agenda');
        agendaTasks = await res.json();
      }
    } else if (name === 'repo') {
      showRepoMap = !showRepoMap;
      if (showRepoMap) {
        const res = await fetch('/api/symbols');
        repoSymbols = await res.json();
      }
    } else if (name === 'diag') {
      showDiagnostics = !showDiagnostics;
    }
  }

  function handleGlobalKeydown(e) {
    if (e.ctrlKey && e.key === 'b') { e.preventDefault(); toggleDrawer('agenda'); }
    if (e.ctrlKey && e.key === 'm') { e.preventDefault(); toggleDrawer('repo'); }
    if (e.ctrlKey && e.key === 'd') { e.preventDefault(); toggleDrawer('diag'); }
    if (e.key === 'Escape') {
      showAgenda = false;
      showRepoMap = false;
      showDiagnostics = false;
      showCompletion = false;
    }
  }
</script>

<div class="app-container">
  <!-- TOP STATUS TELEMETRY -->
  <header>
    <div class="telemetry-group">
      <div class="brand-badge"><Sparkles size={14} color="#7aa2f7" /> pseudo-harness</div>
      <span class="badge engine"><Cpu size={14} /> ENGINE: {model} ({tps} t/s)</span>
      <span class="badge vram"><HardDrive size={14} /> VRAM: {vram}</span>
      <span class="badge tick"><Activity size={14} /> CRDT TICK: {lamportTick}</span>
    </div>
    <div class="token-meter">
      <span class="meter-label"><Gauge size={14} /> BUDGET:</span>
      <div class="token-bar-bg">
        <div class={tokenBarClass} style="width: {tokenPct}%"></div>
      </div>
      <span class="token-text">{tokenCount} / {tokenCeiling} ({tokenPct}%)</span>
    </div>
  </header>

  <!-- 4-REGION WORKBENCH -->
  <div class="workbench">
    <!-- REGION 1: FILE & AST EXPLORER -->
    <div class="region file-tree">
      <div class="region-title">
        <span><FolderTree size={14} /> Workspace Explorer</span>
        <span class="workspace-label">AST INDEX</span>
      </div>
      <ul class="file-list">
        {#each files as file, i}
          <li class="file-item {i === selectedFileIdx ? 'active' : ''}" on:click={() => loadFile(file, i)}>
            {#if file.endsWith('.pseudo')}
              <FileCode size={14} />
            {:else if file.endsWith('.pir')}
              <Cpu size={14} />
            {:else if file.endsWith('.json')}
              <CheckSquare size={14} />
            {:else if file.endsWith('.toml')}
              <Box size={14} />
            {:else}
              <FileText size={14} />
            {/if}
            {file}
          </li>
        {/each}
      </ul>
    </div>

    <!-- REGION 2: ACTIVE EDITOR -->
    <div class="region editor-region">
      <div class="region-title">
        <span><Code size={14} /> {activeFile}</span>
        <button class="action-btn btn-secondary" on:click={runSandbox}><Play size={13} /> Run In Sandbox</button>
      </div>
      <div class="editor-wrapper">
        <textarea class="editor-body" bind:value={editorContent} spellcheck="false"></textarea>
      </div>
    </div>

    <!-- REGION 3: AI STREAM & EXECUTION -->
    <div class="region stream-region">
      <div class="region-title">
        <span><Bot size={14} /> psudo-engine Stream</span>
        <span class="live-badge"><Zap size={12} /> LIVE FFI</span>
      </div>
      <div class="stream-output">
        {#each streamLogs as log}
          {#if log.type === 'user'}
            <div class="msg-bubble user"><Terminal size={14} /> <strong>Operator:</strong> {log.text}</div>
          {:else if log.type === 'talk'}
            <div class="msg-bubble bot"><Bot size={14} /> <pre>{log.text}</pre></div>
          {:else if log.type === 'error'}
            <div class="msg-bubble error">{log.text}</div>
          {:else if log.type === 'success'}
            <div class="msg-bubble success"><ShieldCheck size={14} /> {log.text}</div>
          {:else if log.type === 'warn'}
            <div class="msg-bubble warn">{log.text}</div>
          {/if}
        {/each}
      </div>
    </div>

    <!-- REGION 4: OPERATOR PROMPT -->
    <div class="region prompt-region">
      <div class="region-title">
        <span><Terminal size={14} /> Operator Directive Prompt (Insert Mode)</span>
        <span style="color:var(--emerald); font-weight:600;">Est: 35 Tokens</span>
      </div>
      <div class="prompt-container">
        <div class="prompt-input-wrapper">
          <textarea
            class="prompt-input"
            bind:value={promptInput}
            on:input={handlePromptInput}
            on:keydown={handlePromptKeydown}
            placeholder="Type directive... use '@' for symbols & files (e.g., @net::client::fetch)"
          ></textarea>

          {#if showCompletion && candidates.length > 0}
            <div class="autocomplete-popup show">
              <div class="autocomplete-header"><AtSign size={12} /> In-Context References (@)</div>
              <ul class="autocomplete-list">
                {#each candidates as item, idx}
                  <li class="autocomplete-item {idx === selectedCandidateIdx ? 'selected' : ''}" on:click={() => selectCandidate(item)}>
                    <span style="display:inline-flex;align-items:center;gap:6px;">
                      {#if item.kind === 'file'}
                        <FileCode size={14} />
                      {:else if item.kind === 'struct'}
                        <Box size={14} />
                      {:else}
                        <Code size={14} />
                      {/if}
                      {item.name}
                    </span>
                    <span style="color:var(--text-dim)">[{item.kind}] {item.token_cost}t</span>
                  </li>
                {/each}
              </ul>
            </div>
          {/if}
        </div>

        <div class="prompt-controls">
          <div class="drawer-btns">
            <button class="action-btn btn-secondary" on:click={() => toggleDrawer('agenda')}><ListTodo size={13} /> Agenda (^B)</button>
            <button class="action-btn btn-secondary" on:click={() => toggleDrawer('repo')}><GitMerge size={13} /> Repo Map (^M)</button>
            <button class="action-btn btn-secondary" on:click={() => toggleDrawer('diag')}><ShieldAlert size={13} /> Diagnostics (^D)</button>
          </div>
          <button class="action-btn primary" on:click={submitPrompt}><Send size={13} /> Submit Directive (Enter)</button>
        </div>
      </div>
    </div>
  </div>

  <!-- MODAL DRAWERS -->
  {#if showAgenda}
    <div class="drawer-overlay active" on:click={() => showAgenda = false}>
      <div class="drawer-card" on:click|stopPropagation>
        <div class="drawer-header">
          <span><ListTodo size={16} /> Hierarchical Agenda (@todo)</span>
          <button class="action-btn btn-secondary" on:click={() => showAgenda = false}><X size={14} /></button>
        </div>
        <div class="drawer-content">
          {#each agendaTasks as t}
            <div style="padding:10px 0; border-bottom:1px solid #292e42; display:flex; align-items:center; gap:8px;">
              <CheckCircle2 size={16} color="var(--emerald)" />
              <strong>[{t.status.toUpperCase()}]</strong> {t.title}
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}

  {#if showRepoMap}
    <div class="drawer-overlay active" on:click={() => showRepoMap = false}>
      <div class="drawer-card" on:click|stopPropagation>
        <div class="drawer-header">
          <span><GitMerge size={16} /> Repository Symbol Map</span>
          <button class="action-btn btn-secondary" on:click={() => showRepoMap = false}><X size={14} /></button>
        </div>
        <div class="drawer-content">
          {#each repoSymbols as s}
            <div style="padding:8px 0; border-bottom:1px solid #292e42; color:var(--cyan); display:flex; align-items:center; gap:8px;">
              <Box size={14} /> <strong>{s.name}</strong> <span style="color:var(--text-dim)">[{s.kind}]</span>
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}

  {#if showDiagnostics}
    <div class="drawer-overlay active" on:click={() => showDiagnostics = false}>
      <div class="drawer-card" on:click|stopPropagation>
        <div class="drawer-header">
          <span><ShieldAlert size={16} /> VM Diagnostics & Sandbox Error Trace</span>
          <button class="action-btn btn-secondary" on:click={() => showDiagnostics = false}><X size={14} /></button>
        </div>
        <div class="drawer-content" style="line-height: 1.6;">
          <span style="color:var(--emerald)">[VM STATUS: OK]</span><br/>
          Execution Engine: Direct pIR Virtual Machine<br/>
          Jail Root: .pseudo/sandbox/<br/>
          Last Exit Code: 0 (Success)<br/>
          Memory Trap: None<br/>
          CRDT Lamport Clock: In sync (Tick 142)
        </div>
      </div>
    </div>
  {/if}

  <!-- FOOTER -->
  <footer>
    <span style="display:inline-flex;align-items:center;gap:6px;"><Server size={13} /> pseudo-harness v3.1 WebApp | Host Runtime Active on localhost:3000</span>
    <span style="display:inline-flex;align-items:center;gap:6px;"><Command size={13} /> Shortcuts: Enter (Submit) | @ (Reference) | Esc (Close)</span>
  </footer>
</div>

<style>
  :global(*) { box-sizing: border-box; margin: 0; padding: 0; }
  :global(body) {
    background: #121317;
    color: #c0caf5;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, monospace;
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .app-container {
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  header {
    background: #16161e;
    border-bottom: 1px solid #292e42;
    padding: 8px 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 13px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.3);
    z-index: 10;
  }

  .telemetry-group { display: flex; gap: 12px; align-items: center; }
  .brand-badge {
    font-weight: 700;
    color: #7aa2f7;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: #1f2335;
    padding: 4px 10px;
    border-radius: 6px;
    border: 1px solid #414868;
  }
  .badge {
    padding: 4px 10px;
    border-radius: 6px;
    font-weight: 600;
    font-size: 11px;
    background: #1f2335;
    border: 1px solid #292e42;
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .badge.engine { color: #7aa2f7; border-color: #3d59a1; }
  .badge.vram { color: #73daca; border-color: #1f4438; }
  .badge.tick { color: #bb9af7; border-color: #4b3869; }

  .token-meter {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 12px;
    font-weight: bold;
    background: #1f2335;
    padding: 4px 12px;
    border-radius: 6px;
    border: 1px solid #292e42;
  }
  .meter-label { display: inline-flex; align-items: center; gap: 4px; color: #787c99; }
  .token-text { color: #c0caf5; font-family: monospace; }
  .token-bar-bg {
    width: 120px;
    height: 8px;
    background: #16161e;
    border-radius: 4px;
    overflow: hidden;
    border: 1px solid #292e42;
  }
  .token-bar-fill {
    height: 100%;
    width: 28%;
    background: #73daca;
    transition: width 0.3s, background 0.3s;
  }
  .token-bar-fill.amber { background: #e0af68; }
  .token-bar-fill.crimson { background: #f7768e; }

  .workbench {
    flex: 1;
    display: grid;
    grid-template-columns: 260px 1fr 1fr;
    grid-template-rows: 1fr 200px;
    gap: 2px;
    background: #16161e;
    overflow: hidden;
  }

  .region {
    background: #1a1b26;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .region-title {
    background: #1f2335;
    padding: 8px 14px;
    font-size: 11px;
    text-transform: uppercase;
    font-weight: 700;
    letter-spacing: 0.8px;
    color: #7aa2f7;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid #292e42;
  }

  .file-tree {
    grid-row: 1 / 3;
    grid-column: 1;
    background: #16161e;
    border-right: 1px solid #292e42;
  }
  .file-list {
    list-style: none;
    padding: 8px;
    overflow-y: auto;
  }
  .file-item {
    padding: 8px 12px;
    cursor: pointer;
    font-size: 12px;
    display: flex;
    align-items: center;
    gap: 8px;
    color: #787c99;
    border-radius: 6px;
    margin-bottom: 2px;
    transition: background 0.15s, color 0.15s;
    font-family: monospace;
  }
  .file-item:hover {
    background: #24283b;
    color: #c0caf5;
  }
  .file-item.active {
    background: #283457;
    color: #73daca;
    font-weight: 600;
  }

  .editor-region {
    grid-column: 2;
    grid-row: 1;
    border-right: 1px solid #292e42;
  }
  .editor-wrapper {
    flex: 1;
    position: relative;
    background: #121317;
  }
  .editor-body {
    width: 100%;
    height: 100%;
    background: transparent;
    color: #9ece6a;
    font-family: 'JetBrains Mono', monospace;
    font-size: 13px;
    line-height: 1.6;
    padding: 16px;
    border: none;
    outline: none;
    resize: none;
    white-space: pre;
  }

  .stream-region {
    grid-column: 3;
    grid-row: 1;
  }
  .stream-output {
    flex: 1;
    background: #121317;
    padding: 16px;
    font-family: monospace;
    font-size: 13px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .msg-bubble {
    padding: 10px 14px;
    border-radius: 8px;
    font-size: 13px;
    line-height: 1.5;
  }
  .msg-bubble.user {
    background: #1f2335;
    color: #7aa2f7;
    border: 1px solid #3d59a1;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .msg-bubble.bot {
    background: #181924;
    color: #9ece6a;
    border: 1px solid #292e42;
    border-left: 3px solid #9ece6a;
  }
  .msg-bubble.bot pre {
    white-space: pre-wrap;
    font-family: monospace;
    margin-top: 4px;
  }
  .msg-bubble.error { background: #3d1c24; color: #f7768e; border: 1px solid #5a2631; }
  .msg-bubble.success { background: #18352e; color: #73daca; border: 1px solid #234d43; display: flex; align-items: center; gap: 8px; }
  .msg-bubble.warn { background: #3b2e1e; color: #e0af68; border: 1px solid #57432a; }

  .prompt-region {
    grid-column: 2 / 4;
    grid-row: 2;
    position: relative;
    background: #1f2335;
    border-top: 1px solid #292e42;
  }
  .prompt-container {
    display: flex;
    flex-direction: column;
    flex: 1;
    padding: 12px;
    gap: 10px;
  }
  .prompt-input-wrapper {
    position: relative;
    flex: 1;
  }
  textarea.prompt-input {
    width: 100%;
    height: 100%;
    background: #121317;
    border: 1px solid #414868;
    border-radius: 6px;
    color: #ff9e64;
    padding: 12px;
    font-family: monospace;
    font-size: 13px;
    resize: none;
    outline: none;
    box-shadow: inset 0 2px 4px rgba(0,0,0,0.4);
  }
  textarea.prompt-input:focus { border-color: #7aa2f7; box-shadow: inset 0 2px 4px rgba(0,0,0,0.4), 0 0 0 2px rgba(122,162,247,0.2); }

  .prompt-controls {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .drawer-btns { display: flex; gap: 6px; }
  .action-btn {
    background: #3d59a1;
    border: none;
    color: white;
    padding: 6px 14px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    transition: background 0.15s, transform 0.1s;
  }
  .action-btn:hover { background: #7aa2f7; color: #121317; transform: translateY(-1px); }
  .action-btn.primary { background: #2d4f67; border: 1px solid #7aa2f7; color: #7aa2f7; }
  .action-btn.primary:hover { background: #7aa2f7; color: #121317; }
  .btn-secondary { background: #24283b; border: 1px solid #414868; color: #c0caf5; }
  .btn-secondary:hover { background: #2f354a; color: white; transform: none; }

  .autocomplete-popup {
    position: absolute;
    bottom: calc(100% - 10px);
    left: 20px;
    width: 380px;
    background: #16161e;
    border: 1px solid #7aa2f7;
    border-radius: 8px;
    box-shadow: 0 12px 32px rgba(0,0,0,0.7);
    z-index: 100;
    overflow: hidden;
    display: none;
  }
  .autocomplete-popup.show { display: block; }
  .autocomplete-header {
    background: #1f2335;
    padding: 8px 12px;
    font-size: 11px;
    font-weight: 700;
    color: #7aa2f7;
    border-bottom: 1px solid #292e42;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .autocomplete-list { list-style: none; max-height: 180px; overflow-y: auto; }
  .autocomplete-item {
    padding: 8px 12px;
    font-size: 12px;
    display: flex;
    justify-content: space-between;
    cursor: pointer;
    font-family: monospace;
    align-items: center;
  }
  .autocomplete-item:hover, .autocomplete-item.selected {
    background: #283457;
    color: #73daca;
  }

  .drawer-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.7);
    backdrop-filter: blur(4px);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 200;
  }
  .drawer-card {
    background: #1f2335;
    width: 600px;
    max-height: 80vh;
    border: 1px solid #7aa2f7;
    border-radius: 10px;
    box-shadow: 0 20px 40px rgba(0,0,0,0.8);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .drawer-header {
    padding: 14px 18px;
    background: #16161e;
    border-bottom: 1px solid #292e42;
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-weight: bold;
    color: #7aa2f7;
  }
  .drawer-content { padding: 18px; overflow-y: auto; font-family: monospace; font-size: 13px; color: #c0caf5; }

  footer {
    background: #16161e;
    border-top: 1px solid #292e42;
    padding: 6px 16px;
    font-size: 11px;
    color: #787c99;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .workspace-label { font-size: 10px; color: #787c99; font-weight: 600; }
  .live-badge { color: #73daca; display: inline-flex; align-items: center; gap: 4px; font-weight: 600; }
</style>
