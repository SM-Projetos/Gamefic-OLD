import { invoke } from "@tauri-apps/api/core";

export type Prefs = {
  theme: "dark" | "light" | "auto";
  reducedMotion: boolean;
};

const DEFAULT_PREFS: Prefs = { theme: "dark", reducedMotion: false };

export async function loadPrefs(): Promise<Prefs> {
  try {
    const raw = await invoke<string | null>("get_setting", { key: "prefs" });
    if (raw) {
      const parsed = JSON.parse(raw) as Partial<Prefs>;
      return {
        theme: parsed.theme === "light" || parsed.theme === "auto" ? parsed.theme : "dark",
        reducedMotion: parsed.reducedMotion === true,
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
