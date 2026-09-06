const EMOTION_CLASS: Record<string, string> = {
  Raiva: "text-app-err",
  Feliz: "text-app-success",
  Neutro: "text-app-text",
  Triste: "text-app-warn",
};

type EmotionDebugProps = {
  emotionPt: string | null;
};

export default function EmotionDebug({ emotionPt }: EmotionDebugProps) {
  if (!import.meta.env.DEV || emotionPt == null) {
    return null;
  }

  return (
    <p
      className={`text-center text-2xl font-semibold tracking-tight ${EMOTION_CLASS[emotionPt] ?? "text-app-text"}`}
    >
      {emotionPt}
    </p>
  );
}
