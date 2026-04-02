import { register, init, getLocaleFromNavigator, locale } from "svelte-i18n";
import { saveSetting, getSetting } from "./settings.js";
import nl from "../locales/nl.json";

// Register Dutch synchronously (default), others lazy
register("nl", () => Promise.resolve(nl));
register("en", () => import("../locales/en.json"));
register("de", () => import("../locales/de.json"));

// Initial locale from settings cache (set by App.svelte before this runs),
// fallback to browser language, then Dutch
const initial = getSetting("locale") || getLocaleFromNavigator()?.split("-")[0] || "nl";

init({
  fallbackLocale: "nl",
  initialLocale: initial,
});

// Persist locale choice to config file
locale.subscribe((val) => {
  if (val) saveSetting("locale", val);
});
