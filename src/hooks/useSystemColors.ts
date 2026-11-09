import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SystemColors } from "../types/systemColors.types";
import { applyAccent, applyBackground } from "../utils/systemColors";
import { handleCommandError } from "../utils/error.utils";

/** Fond = couleur de la barre des tâches, accent = accent de Windows (thème et orange par défaut sinon). */
export const useSystemColors = () => {
  useEffect(() => {
    let taskbar: string | null = null;
    const apply = (colors: SystemColors) => {
      taskbar = colors.taskbar;
      applyBackground(colors.taskbar);
      applyAccent(colors.accent);
    };

    invoke<SystemColors>("get_system_colors").then(apply).catch(handleCommandError("get_system_colors"));
    const unlisten = listen<SystemColors>("system-colors", (event) => apply(event.payload));

    const media = window.matchMedia("(prefers-color-scheme: light)");
    const onThemeChange = () => applyBackground(taskbar);
    media.addEventListener("change", onThemeChange);

    return () => {
      unlisten.then((fn) => fn());
      media.removeEventListener("change", onThemeChange);
    };
  }, []);
};
