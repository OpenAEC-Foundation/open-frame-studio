const validators = {
  locale: (value) => typeof value === "string" && /^[a-z]{2}$/.test(value),
  theme: (value) => ["default", "light", "dark", "blue", "amber-navy", "warm-ember", "highContrast"].includes(value),
  left_panel_width: (value) => Number.isFinite(value) && value >= 140 && value <= 400,
  right_panel_width: (value) => Number.isFinite(value) && value >= 200 && value <= 500,
  left_panel_open: (value) => typeof value === "boolean",
  right_panel_open: (value) => typeof value === "boolean",
  ribbon_layout: (value) => value && typeof value.minimized === "boolean" &&
    [value.hiddenTabs, value.hiddenGroups].every((items) =>
      Array.isArray(items) && items.length <= 100 && items.every((s) => typeof s === "string" && s.length <= 80)),
};

export function canStoreWorkspaceSetting(key, value) {
  return Object.hasOwn(validators, key) && validators[key](value);
}

export async function readWorkspaceSettings(settings) {
  if (!settings?.get) return {};
  const entries = await Promise.all(Object.keys(validators).map(async (key) => {
    const value = await settings.get("frame." + key);
    return [key, value];
  }));
  return Object.fromEntries(entries.filter(([key, value]) => canStoreWorkspaceSetting(key, value)));
}

export function hostPreferences(host, locales) {
  const language = typeof host?.taal === "string" ? host.taal.toLowerCase().split("-")[0] : null;
  return {
    locale: locales.includes(language) ? language : null,
    theme: ["dark", "light"].includes(host?.thema) ? host.thema : null,
  };
}
