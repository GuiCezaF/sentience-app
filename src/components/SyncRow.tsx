import type { SyncLine } from "../types/agent";

const SYNC_LABEL: Record<SyncLine, string> = {
  pendente: "Pendente",
  ok: "Concluída",
  erro: "Falhou",
};

type SyncRowProps = {
  sync: SyncLine;
};

export default function SyncRow({ sync }: SyncRowProps) {
  return (
    <div className="group">
      <div className="row">
        <span>Sincronização</span>
        <span className="row-value" data-tone={sync}>
          {SYNC_LABEL[sync]}
        </span>
      </div>
    </div>
  );
}
