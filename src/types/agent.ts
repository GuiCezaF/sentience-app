export type TrayStatus = "Ativo" | "Sem recorte" | "Falha";
export type SyncLine = "pendente" | "ok" | "erro";

export type TraySnapshot = {
  status: TrayStatus;
  sync: SyncLine;
  emotion_pt: string | null;
};

export const INITIAL_SNAPSHOT: TraySnapshot = {
  status: "Sem recorte",
  sync: "pendente",
  emotion_pt: null,
};
