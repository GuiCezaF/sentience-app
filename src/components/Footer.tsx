import { invoke } from "@tauri-apps/api/core";

async function closeApp() {
  await invoke("quit_app");
}

async function logout(): Promise<void> {
  await invoke("logout");
}

export default function Footer() {
  return (
    <div className="group">
      <button type="button" className="row row-button" onClick={() => logout()}>
        Sair da conta
      </button>
      <button
        type="button"
        className="row row-button"
        data-tone="destructive"
        onClick={() => closeApp()}
      >
        Fechar app
      </button>
    </div>
  );
}
