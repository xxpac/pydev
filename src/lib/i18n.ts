import { addMessages, init } from "svelte-i18n";

import en from "../locales/en.json";
import zhCN from "../locales/zh-CN.json";

let started = false;

/// Register the bundled dictionaries synchronously so `$_` is usable on the
/// first render. New languages only need another `addMessages` line + JSON file.
export function setupI18n(initialLocale = "zh-CN"): void {
  if (started) return;
  started = true;
  addMessages("en", en);
  addMessages("zh-CN", zhCN);
  init({
    fallbackLocale: "zh-CN",
    initialLocale,
  });
}
