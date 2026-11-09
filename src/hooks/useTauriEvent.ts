import { useEffect, useRef } from "react";

import { listen } from "@tauri-apps/api/event";

/** Écoute un événement du back pendant la vie du composant ; `handler` peut changer sans se réabonner. */
export const useTauriEvent = <T>(event: string, handler: (payload: T) => void) => {
  const handlerRef = useRef(handler);
  useEffect(() => {
    handlerRef.current = handler;
  });

  useEffect(() => {
    const unlisten = listen<T>(event, (e) => handlerRef.current(e.payload));
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [event]);
};
