<script lang="ts">
  import { _ } from "svelte-i18n";
  import { appState, persistConfig, setLanguage } from "../state.svelte";

  const cfg = $derived(appState.config!);
  let saved = $state(false);

  async function save(): Promise<void> {
    await persistConfig();
    saved = true;
    setTimeout(() => (saved = false), 1500);
  }
</script>

<div class="screen">
  <h2>{$_("settings.title")}</h2>
  <p class="lead">{$_("settings.lead")}</p>

  <div class="card">
    <div class="field">
      <label for="lang">{$_("settings.language")}</label>
      <select
        id="lang"
        value={cfg.language}
        onchange={(e) => setLanguage(e.currentTarget.value)}
      >
        <option value="zh-CN">{$_("settings.chinese")}</option>
        <option value="en">{$_("settings.english")}</option>
      </select>
    </div>

    <p class="hint">{$_("settings.configNote")}</p>

    <button class="btn primary" onclick={save}>
      {saved ? $_("common.saved") : $_("settings.save")}
    </button>
  </div>
</div>
