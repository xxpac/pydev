<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { _ } from "svelte-i18n";

  import { api } from "./lib/api";
  import { appState, pushLog, refreshStatus, setLanguage } from "./lib/state.svelte";
  import type { LogLine, Stage } from "./lib/types";

  import OneClick from "./lib/components/OneClick.svelte";
  import Components from "./lib/components/Components.svelte";
  import Network from "./lib/components/Network.svelte";
  import PathScreen from "./lib/components/PathScreen.svelte";
  import Settings from "./lib/components/Settings.svelte";
  import LogConsole from "./lib/components/LogConsole.svelte";

  type TabId = "oneclick" | "components" | "network" | "path" | "settings";
  const tabs: TabId[] = ["oneclick", "components", "network", "path", "settings"];

  let active = $state<TabId>("oneclick");
  let ready = $state(false);

  onMount(async () => {
    const [os, cfg] = await Promise.all([
      api.detectPlatform(),
      api.getConfig(),
    ]);
    appState.os = os;
    appState.config = cfg;
    await setLanguage(cfg.language || "zh-CN");
    ready = true;

    // Detect what's already installed (with a network check for uv's latest).
    void refreshStatus(true);

    await listen<LogLine>("install://log", (e) => pushLog(e.payload));
    await listen<Stage>("install://progress", (e) => {
      appState.progress = e.payload;
    });
  });
</script>

<div class="app">
  <header class="header">
    <div class="brand">
      <h1>{$_("app.title")}</h1>
      <span class="sub">{$_("app.subtitle")}</span>
    </div>
    <div class="header-right">
      {#if appState.os}
        <span class="badge">{appState.os.os} · {appState.os.arch}</span>
      {/if}
      <div class="lang-toggle">
        <button
          class:active={appState.config?.language === "zh-CN"}
          onclick={() => setLanguage("zh-CN")}>中文</button
        >
        <button
          class:active={appState.config?.language === "en"}
          onclick={() => setLanguage("en")}>EN</button
        >
      </div>
    </div>
  </header>

  {#if ready && appState.config}
    <div class="body">
      <nav class="nav">
        {#each tabs as tab}
          <button class:active={active === tab} onclick={() => (active = tab)}>
            {$_(`nav.${tab}`)}
          </button>
        {/each}
      </nav>

      <main class="content">
        {#if active === "oneclick"}
          <OneClick />
        {:else if active === "components"}
          <Components />
        {:else if active === "network"}
          <Network />
        {:else if active === "path"}
          <PathScreen />
        {:else if active === "settings"}
          <Settings />
        {/if}
      </main>
    </div>

    <LogConsole />
  {/if}
</div>
