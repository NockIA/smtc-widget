import SmtcWidget from "@/components/SmtcWidget/SmtcWidget";
import { useCurrentTrack } from "@/hooks/useCurrentTrack";
import { useWidgetHidden } from "@/hooks/useWidgetHidden";

const App = () => {
  const hidden = useWidgetHidden();
  const { track, receivedAt } = useCurrentTrack(hidden);

  return <SmtcWidget track={track} receivedAt={receivedAt} hidden={hidden} />;
};

export default App;
