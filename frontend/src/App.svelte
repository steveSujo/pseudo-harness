<script>
  import { onMount } from 'svelte';
  import {
    Cpu, HardDrive, Activity, Gauge, FolderTree, FileCode, CheckSquare,
    Box, FileText, Code, Play, Bot, Zap, Terminal, AtSign,
    ListTodo, GitMerge, ShieldAlert, ChevronUp, X, Server, CheckCircle2,
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
  let editorContent = `// pseudo-lang shadcn-svelte design system workbench
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
    { type: "talk", text: "> shadcn-svelte design system components active.\nListening for directive or @ references.\nExecuting pure pIR opcodes." }
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

  // Dialog & Modal States
  let agendaOpen = false;
  let repoOpen = false;
  let diagOpen = false;

  let agendaTasks = [];
  let repoSymbols = [];

  $: tokenPct = Math.min(100, Math.round((tokenCount / tokenCeiling) * 100));
  $: tokenBarVariant = tokenPct > 90 ? "bg-destructive shadow-[0_0_12px_rgba(239,68,68,0.6)]" : tokenPct > 65 ? "bg-amber-500 shadow-[0_0_12px_rgba(245,158,11,0.6)]" : "bg-emerald-500 shadow-[0_0_12px_rgba(16,185,129,0.6)]";

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
    streamLogs = [...streamLogs, { type: 'warn', text: '[SANDBOX EVAL] Validating pIR AST in shadcn design system sandbox...' }];
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

  async function openAgenda() {
    agendaOpen = true;
    const res = await fetch('/api/agenda');
    agendaTasks = await res.json();
  }

  async function openRepo() {
    repoOpen = true;
    const res = await fetch('/api/symbols');
    repoSymbols = await res.json();
  }

  function openDiag() {
    diagOpen = true;
  }

  function handleGlobalKeydown(e) {
    if (e.ctrlKey && e.key === 'b') { e.preventDefault(); openAgenda(); }
    if (e.ctrlKey && e.key === 'm') { e.preventDefault(); openRepo(); }
    if (e.ctrlKey && e.key === 'd') { e.preventDefault(); openDiag(); }
    if (e.key === 'Escape') {
      showCompletion = false;
      agendaOpen = false;
      repoOpen = false;
      diagOpen = false;
    }
  }
</script>

<!-- SHADCN-SVELTE DESIGN SYSTEM APP CONTAINER -->
<div class="h-screen w-screen flex flex-col bg-background text-foreground font-sans antialiased overflow-hidden select-none">
  <!-- TOP HEADER (Card, Badge, Progress primitives) -->
  <header class="h-14 border-b border-border bg-card/90 backdrop-blur px-4 flex items-center justify-between z-10 shadow-sm">
    <div class="flex items-center gap-3">
      <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-md bg-secondary border border-border text-primary font-semibold text-xs shadow-sm">
        <Sparkles size={14} />
        <span>pseudo-harness v3.1</span>
      </div>
      <div class="h-4 w-px bg-border"></div>
      <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-background border border-border text-cyan-400 text-xs font-mono">
        <Cpu size={13} /> {model} ({tps} t/s)
      </span>
      <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-background border border-border text-emerald-400 text-xs font-mono">
        <HardDrive size={13} /> {vram}
      </span>
      <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-background border border-border text-purple-400 text-xs font-mono">
        <Activity size={13} /> Tick: {lamportTick}
      </span>
    </div>

    <!-- Progress Component & Badge -->
    <div class="flex items-center gap-3 bg-background border border-border px-3 py-1.5 rounded-lg shadow-inner">
      <span class="text-xs font-medium text-muted-foreground flex items-center gap-1.5"><Gauge size={14} class="text-primary" /> Token Budget</span>
      <div class="w-32 h-2.5 bg-muted rounded-full overflow-hidden border border-border">
        <div class="h-full transition-all duration-300 {tokenBarVariant}" style="width: {tokenPct}%"></div>
      </div>
      <span class="text-xs font-mono text-foreground font-bold">{tokenCount} / {tokenCeiling}</span>
    </div>
  </header>

  <!-- 4-REGION WORKBENCH GRID -->
  <div class="flex-1 grid grid-cols-[260px_1fr_1fr] grid-rows-[1fr_200px] gap-px bg-border overflow-hidden">
    <!-- REGION 1: EXPLORER -->
    <div class="row-span-2 col-start-1 bg-background flex flex-col border-r border-border">
      <div class="h-10 px-3 bg-card/60 border-b border-border flex items-center justify-between text-xs font-semibold text-muted-foreground tracking-wider">
        <span class="flex items-center gap-2 text-foreground"><FolderTree size={14} class="text-primary" /> Explorer</span>
        <span class="text-[10px] text-muted-foreground font-mono px-1.5 py-0.5 rounded bg-muted border border-border">AST INDEX</span>
      </div>
      <div class="flex-1 overflow-y-auto p-2 flex flex-col gap-1">
        {#each files as file, i}
          <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
          <!-- svelte-ignore a11y-click-events-have-key-events -->
          <div class="flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-mono cursor-pointer transition-all {i === selectedFileIdx ? 'bg-primary/15 text-primary border border-primary/30 font-medium shadow-sm' : 'text-muted-foreground hover:bg-muted hover:text-foreground'}" on:click={() => loadFile(file, i)}>
            {#if file.endsWith('.pseudo')}
              <FileCode size={14} class="text-primary" />
            {:else if file.endsWith('.pir')}
              <Cpu size={14} class="text-purple-400" />
            {:else if file.endsWith('.json')}
              <CheckSquare size={14} class="text-amber-400" />
            {:else if file.endsWith('.toml')}
              <Box size={14} class="text-emerald-400" />
            {:else}
              <FileText size={14} class="text-muted-foreground" />
            {/if}
            <span class="truncate">{file}</span>
          </div>
        {/each}
      </div>
    </div>

    <!-- REGION 2: CODE EDITOR -->
    <div class="col-start-2 row-start-1 bg-background flex flex-col border-r border-border">
      <div class="h-10 px-4 bg-card/60 border-b border-border flex items-center justify-between text-xs font-mono text-foreground">
        <span class="flex items-center gap-2 font-semibold"><Code size={14} class="text-emerald-400" /> {activeFile}</span>
        <button class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-secondary hover:bg-secondary/80 text-secondary-foreground text-xs font-medium border border-border shadow transition-all active:scale-95" on:click={runSandbox}>
          <Play size={12} class="text-emerald-400" />
          <span>Run Sandbox</span>
        </button>
      </div>
      <div class="flex-1 relative bg-background">
        <textarea class="w-full h-full bg-transparent text-emerald-400 font-mono text-xs leading-relaxed p-4 border-none outline-none resize-none selection:bg-emerald-500/30" bind:value={editorContent} spellcheck="false"></textarea>
      </div>
    </div>

    <!-- REGION 3: STREAM OUTPUT -->
    <div class="col-start-3 row-start-1 bg-background flex flex-col">
      <div class="h-10 px-4 bg-card/60 border-b border-border flex items-center justify-between text-xs font-mono text-foreground">
        <span class="flex items-center gap-2 font-semibold"><Bot size={14} class="text-primary" /> psudo-engine Stream</span>
        <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-emerald-500/10 text-emerald-400 text-[10px] font-bold border border-emerald-500/20"><Zap size={12} /> LIVE FFI</span>
      </div>
      <div class="flex-1 p-4 overflow-y-auto flex flex-col gap-3 font-mono text-xs">
        {#each streamLogs as log}
          {#if log.type === 'user'}
            <div class="p-3.5 rounded-xl bg-card border border-border text-primary flex items-start gap-2.5 shadow">
              <Terminal size={14} class="mt-0.5 shrink-0" />
              <div><strong>Operator:</strong> {log.text}</div>
            </div>
          {:else if log.type === 'talk'}
            <div class="p-3.5 rounded-xl bg-card/80 border border-border border-l-4 border-l-emerald-500 text-emerald-400 flex flex-col gap-1.5 shadow">
              <div class="flex items-center gap-2 text-muted-foreground text-[11px]"><Bot size={13} /> Model Response</div>
              <pre class="whitespace-pre-wrap font-mono text-xs leading-relaxed">{log.text}</pre>
            </div>
          {:else if log.type === 'error'}
            <div class="p-3.5 rounded-xl bg-destructive/20 border border-destructive/50 text-destructive-foreground shadow">{log.text}</div>
          {:else if log.type === 'success'}
            <div class="p-3.5 rounded-xl bg-emerald-950/40 border border-emerald-900/50 text-emerald-400 flex items-center gap-2.5 shadow"><ShieldCheck size={16} /> {log.text}</div>
          {:else if log.type === 'warn'}
            <div class="p-3.5 rounded-xl bg-amber-950/40 border border-amber-900/50 text-amber-400 shadow">{log.text}</div>
          {/if}
        {/each}
      </div>
    </div>

    <!-- REGION 4: PROMPT COMMAND BAR -->
    <div class="col-span-2 col-start-2 row-start-2 bg-card flex flex-col border-t border-border relative">
      <div class="h-9 px-4 bg-card/90 border-b border-border flex items-center justify-between text-xs font-mono text-muted-foreground">
        <span class="flex items-center gap-2 font-semibold text-foreground"><Terminal size={14} class="text-amber-400" /> Operator Directive Prompt (Insert Mode)</span>
        <span class="text-emerald-400 font-bold text-[11px]">Est: 35 Tokens</span>
      </div>
      <div class="flex-1 p-3 flex flex-col gap-2.5 relative">
        <div class="flex-1 relative">
          <textarea
            class="w-full h-full bg-background border border-border rounded-xl text-amber-200 p-3.5 font-mono text-xs outline-none focus:border-primary focus:ring-2 focus:ring-primary/30 resize-none shadow-inner"
            bind:value={promptInput}
            on:input={handlePromptInput}
            on:keydown={handlePromptKeydown}
            placeholder="Type directive... use '@' for symbols & files (e.g., @net::client::fetch)"
          ></textarea>

          <!-- Autocomplete Popover Component -->
          {#if showCompletion && candidates.length > 0}
            <div class="absolute bottom-full left-0 mb-2 w-96 bg-card border border-border rounded-xl shadow-2xl overflow-hidden z-50 flex flex-col">
              <div class="px-3 py-2.5 bg-muted border-b border-border text-[11px] font-semibold text-primary flex items-center gap-2">
                <AtSign size={13} /> In-Context References (@)
              </div>
              <ul class="max-h-48 overflow-y-auto p-1 flex flex-col gap-0.5">
                {#each candidates as item, idx}
                  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
                  <!-- svelte-ignore a11y-click-events-have-key-events -->
                  <li class="px-3 py-2 rounded-lg text-xs font-mono flex items-center justify-between cursor-pointer transition-colors {idx === selectedCandidateIdx ? 'bg-primary/20 text-primary-foreground font-medium' : 'text-muted-foreground hover:bg-muted hover:text-foreground'}" on:click={() => selectCandidate(item)}>
                    <span class="flex items-center gap-2.5">
                      {#if item.kind === 'file'}
                        <FileCode size={14} class="text-primary" />
                      {:else if item.kind === 'struct'}
                        <Box size={14} class="text-purple-400" />
                      {:else}
                        <Code size={14} class="text-emerald-400" />
                      {/if}
                      {item.name}
                    </span>
                    <span class="text-[10px] text-muted-foreground">[{item.kind}] {item.token_cost}t</span>
                  </li>
                {/each}
              </ul>
            </div>
          {/if}
        </div>

        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <button class="px-3.5 py-2 rounded-lg bg-secondary hover:bg-secondary/80 text-secondary-foreground text-xs font-medium border border-border shadow inline-flex items-center gap-2 transition-all active:scale-95" on:click={openAgenda}>
              <ListTodo size={14} class="text-amber-400" /> Agenda <kbd class="px-1.5 py-0.5 text-[10px] bg-muted rounded border border-border text-muted-foreground font-mono">^b</kbd>
            </button>
            <button class="px-3.5 py-2 rounded-lg bg-secondary hover:bg-secondary/80 text-secondary-foreground text-xs font-medium border border-border shadow inline-flex items-center gap-2 transition-all active:scale-95" on:click={openRepo}>
              <GitMerge size={14} class="text-purple-400" /> Repo Map <kbd class="px-1.5 py-0.5 text-[10px] bg-muted rounded border border-border text-muted-foreground font-mono">^m</kbd>
            </button>
            <button class="px-3.5 py-2 rounded-lg bg-secondary hover:bg-secondary/80 text-secondary-foreground text-xs font-medium border border-border shadow inline-flex items-center gap-2 transition-all active:scale-95" on:click={openDiag}>
              <ShieldAlert size={14} class="text-red-400" /> Diagnostics <kbd class="px-1.5 py-0.5 text-[10px] bg-muted rounded border border-border text-muted-foreground font-mono">^d</kbd>
            </button>
          </div>
          <button class="px-5 py-2 rounded-lg bg-primary hover:bg-primary/90 text-primary-foreground font-bold text-xs shadow-lg inline-flex items-center gap-2 transition-all active:scale-95" on:click={submitPrompt}>
            <ChevronUp size={15} /> Submit <kbd class="px-1.5 py-0.5 text-[10px] bg-primary-foreground/20 rounded text-primary-foreground font-mono">Enter</kbd>
          </button>
        </div>
      </div>
    </div>
  </div>

  <!-- SHADCN DIALOG COMPONENTS -->
  {#if agendaOpen}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4 animate-in fade-in duration-200" on:click={() => agendaOpen = false}>
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <div class="w-[650px] max-h-[80vh] bg-card border border-border rounded-2xl shadow-2xl flex flex-col overflow-hidden" on:click|stopPropagation>
        <div class="px-6 py-4 bg-muted border-b border-border flex items-center justify-between font-semibold text-primary">
          <span class="flex items-center gap-2 text-sm"><ListTodo size={18} /> Hierarchical Agenda (@todo)</span>
          <button class="p-1.5 rounded-lg hover:bg-secondary text-muted-foreground hover:text-foreground transition-colors" on:click={() => agendaOpen = false}><X size={16} /></button>
        </div>
        <div class="p-6 overflow-y-auto flex flex-col gap-3 font-mono text-xs">
          {#each agendaTasks as t}
            <div class="p-3.5 rounded-xl bg-background border border-border flex items-center justify-between shadow-sm">
              <span class="flex items-center gap-3">
                <CheckCircle2 size={16} class="text-emerald-400" />
                <span class="text-foreground font-medium">{t.title}</span>
              </span>
              <span class="px-2.5 py-1 rounded-md text-[10px] font-bold bg-secondary text-primary border border-border">{t.status.toUpperCase()}</span>
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}

  {#if repoOpen}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4 animate-in fade-in duration-200" on:click={() => repoOpen = false}>
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <div class="w-[650px] max-h-[80vh] bg-card border border-border rounded-2xl shadow-2xl flex flex-col overflow-hidden" on:click|stopPropagation>
        <div class="px-6 py-4 bg-muted border-b border-border flex items-center justify-between font-semibold text-primary">
          <span class="flex items-center gap-2 text-sm"><GitMerge size={18} /> Repository Symbol Map</span>
          <button class="p-1.5 rounded-lg hover:bg-secondary text-muted-foreground hover:text-foreground transition-colors" on:click={() => repoOpen = false}><X size={16} /></button>
        </div>
        <div class="p-6 overflow-y-auto flex flex-col gap-2.5 font-mono text-xs">
          {#each repoSymbols as s}
            <div class="p-3 rounded-xl bg-background border border-border flex items-center justify-between text-primary shadow-sm">
              <span class="flex items-center gap-3">
                <Box size={14} class="text-purple-400" />
                <strong class="font-semibold">{s.name}</strong>
              </span>
              <span class="text-muted-foreground text-[10px] px-2 py-0.5 rounded bg-muted border border-border">[{s.kind}]</span>
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}

  {#if diagOpen}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4 animate-in fade-in duration-200" on:click={() => diagOpen = false}>
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <div class="w-[650px] max-h-[80vh] bg-card border border-border rounded-2xl shadow-2xl flex flex-col overflow-hidden" on:click|stopPropagation>
        <div class="px-6 py-4 bg-muted border-b border-border flex items-center justify-between font-semibold text-primary">
          <span class="flex items-center gap-2 text-sm"><ShieldAlert size={18} /> VM Diagnostics & Sandbox Error Trace</span>
          <button class="p-1.5 rounded-lg hover:bg-secondary text-muted-foreground hover:text-foreground transition-colors" on:click={() => diagOpen = false}><X size={16} /></button>
        </div>
        <div class="p-6 overflow-y-auto font-mono text-xs text-muted-foreground flex flex-col gap-2.5 bg-background leading-relaxed shadow-inner">
          <div class="text-emerald-400 font-bold flex items-center gap-2"><ShieldCheck size={14} /> [VM STATUS: OK]</div>
          <div>Execution Engine: Direct pIR Virtual Machine</div>
          <div>Jail Root: .pseudo/sandbox/</div>
          <div>Last Exit Code: 0 (Success)</div>
          <div>Memory Trap: None</div>
          <div>CRDT Lamport Clock: In sync (Tick 142)</div>
        </div>
      </div>
    </div>
  {/if}

  <!-- FOOTER -->
  <footer class="h-8 bg-card border-t border-border px-4 flex items-center justify-between text-[11px] text-muted-foreground">
    <span class="flex items-center gap-2 font-mono"><Server size={13} class="text-primary" /> pseudo-harness v3.1 Shadcn Design System | localhost:3000</span>
    <div class="flex items-center gap-3 font-mono">
      <span><kbd class="px-1.5 py-0.5 bg-muted rounded border border-border text-foreground shadow-sm">Enter</kbd> Submit</span>
      <span><kbd class="px-1.5 py-0.5 bg-muted rounded border border-border text-foreground shadow-sm">@</kbd> Ref</span>
      <span><kbd class="px-1.5 py-0.5 bg-muted rounded border border-border text-foreground shadow-sm">Esc</kbd> Close</span>
      <span><kbd class="px-1.5 py-0.5 bg-muted rounded border border-border text-foreground shadow-sm">^b</kbd> Agenda</span>
      <span><kbd class="px-1.5 py-0.5 bg-muted rounded border border-border text-foreground shadow-sm">^m</kbd> Repo</span>
      <span><kbd class="px-1.5 py-0.5 bg-muted rounded border border-border text-foreground shadow-sm">^d</kbd> Diag</span>
    </div>
  </footer>
</div>
