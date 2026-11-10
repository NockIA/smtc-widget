import { useCallback, useEffect, useRef } from "react";

import { EVENTS } from "@/constants/EVENTS";
import { useTauriEvent } from "@/hooks/useTauriEvent";
import { SystemColors } from "@/types/systemColors.types";
import { commands } from "@/utils/commands.utils";
import { applyAccent, applyBackground } from "@/utils/systemColors.utils";

/** Fond = couleur de la barre des tâches, accent = accent de Windows (thème et orange par défaut sinon). */
export const useSystemColors = () => {
  const latest = useRef<Pick<SystemColors, "taskbar" | "transparent">>({ taskbar: null, transparent: false });

  const apply = useCallback((colors: SystemColors) => {
    latest.current = colors;
    applyBackground(colors.taskbar, colors.transparent);
    applyAccent(colors.accent);
  }, []);

  useEffect(() => {
    commands.getSystemColors().then((colors) => colors && apply(colors));
  }, [apply]);

  useTauriEvent<SystemColors>(EVENTS.systemColors, apply);

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: light)");
    const onThemeChange = () => applyBackground(latest.current.taskbar, latest.current.transparent);
    media.addEventListener("change", onThemeChange);
    return () => media.removeEventListener("change", onThemeChange);
  }, []);
};
