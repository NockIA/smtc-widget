import { useEffect, useState } from "react";

import { EVENTS } from "@/constants/EVENTS";
import { useTauriEvent } from "@/hooks/useTauriEvent";

/** Vrai tant qu'une application plein écran cache le widget ; la classe `widget-hidden` met alors les animations CSS en pause. */
export const useWidgetHidden = () => {
  const [hidden, setHidden] = useState(false);

  useTauriEvent<boolean>(EVENTS.widgetVisibility, setHidden);

  useEffect(() => {
    document.documentElement.classList.toggle("widget-hidden", hidden);
  }, [hidden]);

  return hidden;
};
