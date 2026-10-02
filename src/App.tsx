import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Status = "offline" | "connecting" | "online";

type BackendStatus = {
  running: boolean;
  binary_found: boolean;
  config_found: boolean;
};

export default function App() {
  const [status, setStatus] = useState<Status>("offline");
  const [backend, setBackend] = useState<BackendStatus | null>(null);
  const [error, setError] = useState("");

  const refreshStatus = async () => {
    try {
      const result = await invoke<BackendStatus>("get_status");
      setBackend(result);
      setStatus(result.running ? "online" : "offline");
    } catch (err) {
      setError(String(err));
      setStatus("offline");
    }
  };

  useEffect(() => {
    void refreshStatus();

    const timer = window.setInterval(() => {
      void refreshStatus();
    }, 1000);

    return () => window.clearInterval(timer);
  }, []);

  const connect = async () => {
    setError("");
    setStatus("connecting");

    try {
      await invoke("connect");
      await refreshStatus();
    } catch (err) {
      setStatus("offline");
      setError(String(err));
    }
  };

  const disconnect = async () => {
    setError("");

    try {
      await invoke("disconnect");
      await refreshStatus();
    } catch (err) {
      setError(String(err));
    }
  };

  const isOnline = status === "online";
  const isConnecting = status === "connecting";
  const binaryReady = backend?.binary_found ?? false;
  const configReady = backend?.config_found ?? false;

  const statusText = isOnline
    ? "Connected"
    : isConnecting
      ? "Connecting..."
      : "Disconnected";

  const subtitle = isOnline
    ? "sing-box is running"
    : error || "Ready to start sing-box";

  return (
    <main className="app">
      <header className="topbar">
        <div className="brand">
          <span className={"brand-mark " + (isOnline ? "active" : "")} />
          <span>PumatVPN</span>
        </div>
        <span className="version">v0.1.0</span>
      </header>

      <section className="content">
        <div className="eyebrow">VPN CONNECTION</div>

        <div className={"status-dot " + status}>
          <span />
        </div>

        <h1>{statusText}</h1>
        <p className={"subtitle" + (error ? " error" : "")}>{subtitle}</p>

        <div className="config-card">
          <div className="config-icon">{"{}"}</div>

          <div className="config-info">
            <strong>config.json</strong>
            <span>Configuration file</span>
          </div>

          <div className="config-state">
            <span className="check">{configReady ? "✓" : "!"}</span>
            {configReady ? "Ready" : "Missing"}
          </div>
        </div>

        <button
          className={"connect-button " + (isOnline ? "disconnect" : "")}
          onClick={() => void (isOnline ? disconnect() : connect())}
          disabled={isConnecting || !binaryReady || !configReady}
        >
          {isConnecting
            ? "Connecting..."
            : isOnline
              ? "Disconnect"
              : "Connect"}
        </button>

        <div className="engine-status">
          <span className="engine-label">Sing-box</span>
          <span className="engine-value">
            <i className={"online-indicator" + (!binaryReady ? " offline" : "")} />
            {binaryReady ? "Binary ready" : "Binary missing"}
          </span>
        </div>
      </section>
    </main>
  );
}
