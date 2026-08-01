<script lang="ts">
  import { _ } from "svelte-i18n";
  import { api } from "../api";
  import { appState, runAction } from "../state.svelte";

  const cfg = $derived(appState.config!);
  const status = $derived(appState.status);
  const exts = $derived(status?.extensions ?? []);
  const extMissing = $derived(exts.filter((e) => !e.installed).length);
</script>

{#snippet stateTag(satisfied: boolean)}
  {#if !status}
    <span class="badge muted">{$_("status.checking")}</span>
  {:else if satisfied && !cfg.reinstall_existing}
    <span class="badge ok">{$_("oneclick.installedTag")}</span>
  {:else if satisfied}
    <span class="badge warn">{$_("oneclick.willUpdate")}</span>
  {:else}
    <span class="badge accent">{$_("oneclick.willInstall")}</span>
  {/if}
{/snippet}

<div class="screen">
  <h2>{$_("oneclick.title")}</h2>
  <p class="lead">{$_("oneclick.lead")}</p>

  <div class="card">
    <h3>{$_("oneclick.summaryTitle")}</h3>
    <ul class="summary">
      <li class="srow">
        <span><span class="dot"></span>{$_("oneclick.uv")}</span>
        {@render stateTag(status?.uv.installed ?? false)}
      </li>
      <li class="srow">
        <span>
          <span class="dot"></span>
          {$_("oneclick.python", { values: { version: cfg.python.version } })}
        </span>
        {@render stateTag(status?.python.satisfied ?? false)}
      </li>
      {#if cfg.vscode.install}
        <li class="srow">
          <span><span class="dot"></span>{$_("oneclick.vscode")}</span>
          {@render stateTag(status?.vscode.installed ?? false)}
        </li>
        <li class="srow">
          <span>
            <span class="dot"></span>
            {$_("oneclick.extensions", {
              values: { count: cfg.vscode.extensions.length },
            })}
          </span>
          {@render stateTag(exts.length > 0 && extMissing === 0)}
        </li>
      {/if}
      <li class="srow">
        <span><span class="dot"></span>{$_("oneclick.path")}</span>
        {@render stateTag(status?.path.configured ?? false)}
      </li>
    </ul>

    <label class="checkline">
      <input type="checkbox" bind:checked={cfg.reinstall_existing} />
      {$_("oneclick.reinstall")}
    </label>
    <p class="hint">{$_("oneclick.skipHint")}</p>

    <button
      class="btn primary big"
      disabled={appState.busy}
      onclick={() => runAction((c) => api.oneClick(c))}
    >
      {appState.busy ? $_("oneclick.running") : $_("oneclick.button")}
    </button>
  </div>
</div>
