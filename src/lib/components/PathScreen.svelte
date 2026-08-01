<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import { api } from "../api";
  import { appState, runAction } from "../state.svelte";
  import type { PathPreview } from "../types";

  const cfg = $derived(appState.config!);
  const isWindows = $derived(appState.os?.os === "windows");
  const shellOptions = ["bashrc", "zshrc", "fish", "profile"];

  let preview = $state<PathPreview | null>(null);

  async function loadPreview(): Promise<void> {
    try {
      preview = await api.pathPreview(appState.config!);
    } catch {
      preview = null;
    }
  }

  onMount(loadPreview);

  function toggleShell(shell: string, on: boolean): void {
    if (on) {
      if (!cfg.path.shells.includes(shell)) cfg.path.shells.push(shell);
    } else {
      cfg.path.shells = cfg.path.shells.filter((s) => s !== shell);
    }
    loadPreview();
  }

  async function apply(): Promise<void> {
    await runAction((c) => api.applyPath(c));
    await loadPreview();
  }
</script>

<div class="screen">
  <h2>{$_("path.title")}</h2>
  <p class="lead">{$_("path.lead")}</p>

  <div class="card">
    {#if preview}
      <div class="field">
        <span class="flabel">{$_("path.entries")}</span>
        <ul class="path-list mono">
          {#each preview.entries as e}
            <li>{e}</li>
          {/each}
        </ul>
      </div>
      <div class="field">
        <span class="flabel">{$_("path.targets")}</span>
        <ul class="path-list mono">
          {#each preview.targets as t}
            <li>{t}</li>
          {/each}
        </ul>
      </div>
    {/if}

    {#if isWindows}
      <p class="hint">{$_("path.windowsNote")}</p>
    {:else}
      <div class="field">
        <span class="flabel">{$_("path.shells")}</span>
        <div class="checks">
          {#each shellOptions as s}
            <label>
              <input
                type="checkbox"
                checked={cfg.path.shells.includes(s)}
                onchange={(e) => toggleShell(s, e.currentTarget.checked)}
              />
              {s}
            </label>
          {/each}
        </div>
      </div>
    {/if}

    <div class="checks" style="margin:6px 0 14px">
      <label>
        <input type="checkbox" bind:checked={cfg.path.update} />
        {$_("path.update")}
      </label>
    </div>

    <button
      class="btn primary"
      disabled={appState.busy || !cfg.path.update}
      onclick={apply}
    >
      {appState.busy ? $_("common.applying") : $_("path.apply")}
    </button>
  </div>
</div>
