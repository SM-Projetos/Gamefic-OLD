export type ThemeChoice = "dark" | "light" | "auto";

const TRANSPARENT_PIXEL =
  "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7";

export function applyTheme(choice: ThemeChoice): void {
  const root = document.documentElement;
  const dark = window.matchMedia("(prefers-color-scheme: dark)");
  const set = () => {
    root.dataset["theme"] = choice === "auto" ? (dark.matches ? "dark" : "light") : choice;
  };
  dark.removeEventListener("change", set);
  if (choice === "auto") dark.addEventListener("change", set);
  set();
}

export function applyReducedMotion(enabled: boolean): void {
  document.documentElement.dataset["reducedMotion"] = String(enabled);
}

export function notify(
  container: HTMLElement,
  kind: "ok" | "error" | "info",
  message: string,
): void {
  const toast = document.createElement("div");
  toast.className = `toast toast--${kind}`;
  toast.textContent = message;
  container.appendChild(toast);
  new Image().src = TRANSPARENT_PIXEL;
  window.setTimeout(() => {
    toast.style.opacity = "0";
    toast.style.transition = "opacity 200ms ease";
    window.setTimeout(() => toast.remove(), 220);
  }, 3200);
}
