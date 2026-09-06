import type { TrayStatus } from "../types/agent";

type StatusPillProps = {
  status: TrayStatus;
};

export default function StatusPill({ status }: StatusPillProps) {
  return (
    <span className="status-pill" data-status={status} aria-live="polite">
      <span className="status-dot" aria-hidden="true" />
      {status}
    </span>
  );
}
