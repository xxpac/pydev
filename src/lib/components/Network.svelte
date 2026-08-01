<script lang="ts">
  import { _ } from "svelte-i18n";
  import { api } from "../api";
  import { appState, runAction } from "../state.svelte";
  import type { EndpointResult } from "../types";

  const cfg = $derived(appState.config!);
  let results = $state<EndpointResult[]>([]);

  const okCount = $derived(results.filter((r) => r.ok).length);

  async function test(): Promise<void> {
    await runAction(async (c) => {
      results = await api.testNetwork(c);
    });
  }
</script>

<div class="screen">
  <h2>{$_("network.title")}</h2>
  <p class="lead">{$_("network.lead")}</p>

  <div class="card">
    <div class="field">
      <label for="http">{$_("network.http")}</label>
      <input id="http" bind:value={cfg.proxy.http} placeholder={$_("network.placeholder")} />
    </div>
    <div class="field">
      <label for="https">{$_("network.https")}</label>
      <input id="https" bind:value={cfg.proxy.https} placeholder={$_("network.placeholder")} />
    </div>
    <div class="field">
      <label for="noproxy">{$_("network.noProxy")}</label>
      <input id="noproxy" bind:value={cfg.proxy.no_proxy} placeholder="localhost,127.0.0.1" />
    </div>

    <div class="row">
      <button class="btn primary" disabled={appState.busy} onclick={test}>
        {appState.busy ? $_("common.testing") : $_("common.test")}
      </button>
    </div>

    {#if results.length > 0}
      <div class="results">
        {#each results as r}
          <div class="result">
            <div>
              <div>{r.name}</div>
              <div class="mono" style="color:var(--text-dim)">{r.url}</div>
            </div>
            <div class="row">
              {#if r.latency_ms !== null}
                <span class="mono" style="color:var(--text-dim)">{r.latency_ms} ms</span>
              {/if}
              <span class="status-pill {r.ok ? 'ok' : 'bad'}">
                {r.ok ? $_("network.reachable") : $_("network.unreachable")}
              </span>
            </div>
          </div>
        {/each}
      </div>
      <p class="hint" style="margin-top:12px">
        {$_("network.summary", { values: { ok: okCount, total: results.length } })}
        {#if okCount < results.length}
          — {$_("network.hint")}
        {/if}
      </p>
    {/if}
  </div>
</div>
