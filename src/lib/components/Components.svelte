<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import { api } from "../api";
  import { appState, refreshStatus, runAction } from "../state.svelte";
  import type { Tool } from "../types";

  const cfg = $derived(appState.config!);
  const status = $derived(appState.status);
  const uv = $derived(status?.uv ?? null);
  const py = $derived(status?.python ?? null);
  const code = $derived(status?.vscode ?? null);
  const exts = $derived(status?.extensions ?? []);
  const extInstalled = $derived(exts.filter((e) => e.installed).length);
  const extOutdated = $derived(exts.filter((e) => e.upgrade_available).length);
  const extMap = $derived(new Map(exts.map((e) => [e.id.toLowerCase(), e])));

  let versions = $state<string[]>([]);
  let newExt = $state("");

  onMount(async () => {
    try {
      versions = await api.listPythonVersions(appState.config!);
    } catch {
      versions = [cfg.python.version];
    }
  });

  function toolLabel(tool: Tool | null): string {
    if (!tool || !tool.installed) return $_("common.install");
    if (tool.upgrade_available && tool.latest) {
      return $_("common.upgradeTo", { values: { version: tool.latest } });
    }
    return $_("common.update");
  }

  function addExt(): void {
    const v = newExt.trim();
    if (v && !cfg.vscode.extensions.includes(v)) {
      cfg.vscode.extensions.push(v);
    }
    newExt = "";
  }

  function removeExt(name: string): void {
    cfg.vscode.extensions = cfg.vscode.extensions.filter((e) => e !== name);
  }
</script>

{#snippet toolBadge(tool: Tool)}
  {#if tool.installed}
    <span class="badge ok"
      >{$_("status.installed")}{tool.current ? ` · ${tool.current}` : ""}</span
    >
    {#if tool.upgrade_available}
      <span class="badge warn"
        >{$_("status.updateAvailable")}{tool.latest
          ? ` · ${tool.latest}`
          : ""}</span
      >
    {/if}
  {:else}
    <span class="badge">{$_("status.notInstalled")}</span>
  {/if}
{/snippet}

<div class="screen">
  <div class="spread">
    <h2>{$_("components.title")}</h2>
    <button
      class="btn ghost small"
      disabled={appState.busy}
      onclick={() => refreshStatus(true)}
    >
      {$_("common.refresh")}
    </button>
  </div>
  <p class="lead">{$_("components.lead")}</p>

  <!-- uv -->
  <div class="card">
    <div class="spread">
      <h3>{$_("components.uv.title")}</h3>
      <div class="badges">
        {#if !status}
          <span class="badge muted">{$_("status.checking")}</span>
        {:else}
          {@render toolBadge(uv!)}
        {/if}
      </div>
    </div>
    <p class="hint">{$_("components.uv.hint")}</p>
    <button
      class="btn primary"
      disabled={appState.busy}
      onclick={() => runAction((c) => api.installUv(c))}
    >
      {toolLabel(uv)}
    </button>
  </div>

  <!-- Python -->
  <div class="card">
    <div class="spread">
      <h3>{$_("components.python.title")}</h3>
      <div class="badges">
        {#if !status}
          <span class="badge muted">{$_("status.checking")}</span>
        {:else if py?.satisfied}
          <span class="badge ok">{$_("status.installed")}</span>
        {:else}
          <span class="badge">{$_("components.python.missing")}</span>
        {/if}
      </div>
    </div>
    <p class="hint">{$_("components.python.hint")}</p>
    {#if py}
      <p class="substatus">
        {py.installed_versions.length
          ? $_("components.python.installedList", {
              values: { versions: py.installed_versions.join(", ") },
            })
          : $_("components.python.none")}
      </p>
    {/if}
    <div class="field">
      <label for="py-version">{$_("components.python.version")}</label>
      <select id="py-version" bind:value={cfg.python.version}>
        {#each versions as v}
          <option value={v}>{v}</option>
        {/each}
      </select>
    </div>
    <div class="checks">
      <label>
        <input type="checkbox" bind:checked={cfg.python.set_default} />
        {$_("components.python.setDefault")}
      </label>
    </div>
    <div class="row" style="margin-top:14px">
      <button
        class="btn primary"
        disabled={appState.busy}
        onclick={() => runAction((c) => api.installPython(c))}
      >
        {$_("common.install")}
      </button>
    </div>
  </div>

  <!-- VSCode -->
  <div class="card">
    <div class="spread">
      <h3>{$_("components.vscode.title")}</h3>
      <div class="badges">
        {#if !status}
          <span class="badge muted">{$_("status.checking")}</span>
        {:else}
          {@render toolBadge(code!)}
        {/if}
        <label class="switch">
          <input type="checkbox" bind:checked={cfg.vscode.install} />
          <span class="slider"></span>
        </label>
      </div>
    </div>
    <p class="hint">{$_("components.vscode.hint")}</p>

    <div class="field">
      <span class="flabel">{$_("components.vscode.extensions")}</span>
      <div class="tags">
        {#each cfg.vscode.extensions as ext}
          {@const est = extMap.get(ext.toLowerCase())}
          <span
            class="tag"
            class:installed={est?.installed}
            class:outdated={est?.upgrade_available}
          >
            {#if est?.upgrade_available}
              <span class="up" title={est.latest ?? ""} aria-hidden="true"
                >↑</span
              >
            {:else if est?.installed}
              <span class="tick" aria-hidden="true">✓</span>
            {/if}
            {ext}
            <button onclick={() => removeExt(ext)} aria-label="remove">×</button>
          </span>
        {/each}
      </div>
      {#if exts.length}
        <p class="substatus">
          {$_("components.vscode.extCount", {
            values: { installed: extInstalled, total: exts.length },
          })}{extOutdated
            ? $_("components.vscode.extUpdates", {
                values: { count: extOutdated },
              })
            : ""}
        </p>
      {/if}
      <div class="row">
        <input
          placeholder={$_("components.vscode.addPlaceholder")}
          bind:value={newExt}
          onkeydown={(e) => e.key === "Enter" && addExt()}
        />
        <button class="btn" onclick={addExt}>{$_("common.add")}</button>
      </div>
    </div>

    <div class="row" style="margin-top:6px">
      <button
        class="btn primary"
        disabled={appState.busy || !cfg.vscode.install}
        onclick={() => runAction((c) => api.installVscode(c))}
      >
        {toolLabel(code)}
      </button>
      <button
        class="btn"
        disabled={appState.busy}
        onclick={() => runAction((c) => api.installExtensions(c))}
      >
        {$_("components.vscode.installExt")}
      </button>
    </div>
  </div>
</div>
