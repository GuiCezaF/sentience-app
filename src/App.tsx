import "./App.css";
import CameraFeed from "./components/CameraFeed";
import EmotionDebug from "./components/EmotionDebug";
import Footer from "./components/Footer";
import Navbar from "./components/Navbar";
import SyncRow from "./components/SyncRow";
import { useAgentSnapshot } from "./hooks/useAgentSnapshot";
import { useCameraSession } from "./hooks/useCameraSession";

const SUBJECT_PLACEHOLDER = "Guilherme";

export default function App() {
  const { snapshot, refresh } = useAgentSnapshot();
  const { stream, cameraState, cameraFailure, cardVisible } = useCameraSession({
    onTick: refresh,
  });

  return (
    <div className="app-shell" data-visible={cardVisible}>
      <Navbar subjectName={SUBJECT_PLACEHOLDER} status={snapshot.status} />

      <main className="app-main">
        <CameraFeed
          stream={stream}
          state={cameraState}
          failure={cameraFailure}
          visible={cardVisible}
        >
          <EmotionDebug emotionPt={snapshot.emotion_pt} />
        </CameraFeed>
        <SyncRow sync={snapshot.sync} />
      </main>

      <Footer />
    </div>
  );
}
