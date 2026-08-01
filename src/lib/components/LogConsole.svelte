<script lang="ts">
  import { _ } from "svelte-i18n";
  import { appState, clearLogs } from "../state.svelte";

  let collapsed = $state(false);
  let bodyEl: HTMLDivElement | undefined = $state();

  const pct = $derived(
    appState.progress
      ? Math.round((appState.progress.index / appState.progress.total) * 100)
      : 0,
  );

  // Auto-scroll to the newest line whenever the log grows.
  $effect(() => {
    void appState.logs.length;
    if (bodyEl && !collapsed) {
      bodyEl.scrollTop = bodyEl.scrollHeight;
    }
  });
</script>

<section class="logpanel" class:collapsed>
  <div
    class="logbar"
    role="button"
    tabindex="0"
    onclick={() => (collapsed = !collapsed)}
    onkeydown={(e) => e.key === "Enter" && (collapsed = !collapsed)}
  >
    <div class="title">
      <span>{$_("log.title")}</span>
      {#if appState.progress}
        <span class="mono">
          {appState.progress.index}/{appState.progress.total} ·
          {$_(`stage.${appState.progress.key}`)}
        </span>
        <span class="progress"><span style="width:{pct}%"></span></span>
      {/if}
    </div>
    <div class="actions">
      <button
        onclick={(e) => {
          e.stopPropagation();
          clearLogs();
        }}>{$_("log.clear")}</button
      >
      <button
        onclick={(e) => {
          e.stopPropagation();
          collapsed = !collapsed;
        }}>{collapsed ? $_("log.show") : $_("log.hide")}</button
      >
    </div>
  </div>

  {#if !collapsed}
    <div class="logbody" bind:this={bodyEl}>
      {#if appState.logs.length === 0}
        <div class="empty">{$_("log.empty")}</div>
      {:else}
        {#each appState.logs as line}
          <div class="logline {line.level}">{line.message}</div>
        {/each}
      {/if}
    </div>
  {/if}
</section>
