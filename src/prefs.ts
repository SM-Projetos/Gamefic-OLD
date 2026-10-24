import { invoke } from "@tauri-apps/api/core";

export type Prefs = {
  theme: "dark" | "light" | "auto";
  reducedMotion: boolean;
  monitorIntervalMs: number;
};

const DEFAULT_PREFS: Prefs = { theme: "dark", reducedMotion: false, monitorIntervalMs: 2000 };

export async function loadPrefs(): Promise<Prefs> {
  try {
    const raw = await invoke<string | null>("get_setting", { key: "prefs" });
    if (raw) {
      const parsed = JSON.parse(raw) as Partial<Prefs>;
      const interval = parsed.monitorIntervalMs;
      return {
        theme: parsed.theme === "light" || parsed.theme === "auto" ? parsed.theme : "dark",
        reducedMotion: parsed.reducedMotion === true,
        monitorIntervalMs:
          typeof interval === "number" && [1000, 2000, 5000].includes(interval) ? interval : 2000,
      };
    }
  } catch (e) {
    console.warn("Não foi possível carregar as preferências:", e);
  }
  return DEFAULT_PREFS;
}

export function savePrefs(prefs: Prefs): void {
  invoke("set_setting", { key: "prefs", value: JSON.stringify(prefs) }).catch((e) =>
    console.warn("Não foi possível salvar as preferências:", e),
  );
}
