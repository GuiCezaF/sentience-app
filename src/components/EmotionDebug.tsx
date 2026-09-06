const EMOTION_COLOR: Record<string, string> = {
  Raiva: "var(--color-red)",
  Feliz: "var(--color-green)",
  Neutro: "rgb(255 255 255 / 0.7)",
  Triste: "var(--color-orange)",
};

type EmotionDebugProps = {
  emotionPt: string | null;
};

export default function EmotionDebug({ emotionPt }: EmotionDebugProps) {
  if (!import.meta.env.DEV || emotionPt == null) {
    return null;
  }

  return (
    <span className="chip">
      <span
        className="chip-dot"
        style={{ background: EMOTION_COLOR[emotionPt] ?? "currentColor" }}
        aria-hidden="true"
      />
      {emotionPt}
    </span>
  );
}
