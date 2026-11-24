import { register, init, getLocaleFromNavigator, locale, locales } from "svelte-i18n";
import { get } from "svelte/store";
import { saveSetting, getSetting } from "./settings.js";
import nl from "../locales/nl.json";

// Register Dutch synchronously (default), others lazy
register("nl", () => Promise.resolve(nl));
register("en", () => import("../locales/en.json"));
register("de", () => import("../locales/de.json"));
register("tr", () => import("../locales/tr.json"));

// Stored preferences and the browser can name a language the app does not ship.
const supported = get(locales);
const initial = [getSetting("locale"), getLocaleFromNavigator()]
  .map(value => typeof value === "string" ? value.toLowerCase().split("-")[0] : null)
  .find(language => supported.includes(language)) || "nl";

init({
  fallbackLocale: "nl",
  initialLocale: initial,
});

// Persist locale choice to config file
locale.subscribe((val) => {
  if (val) saveSetting("locale", val);
});
