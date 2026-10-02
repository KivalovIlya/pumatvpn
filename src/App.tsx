import { useState } from "react";

type Status = "offline" | "connecting" | "online";

export default function App() {
  const [status, setStatus] = useState<Status>("offline");
  const isOnline = status === "online";
  const isConnecting = status === "connecting";
  const toggle = () => {
    if (isOnline || isConnecting) return setStatus("offline");
    setStatus("connecting");
    window.setTimeout(() => setStatus("online"), 700);
  };
  const statusText = isOnline ? "Connected" : isConnecting ? "Connecting..." : "Disconnected";
  return <main className="app">
    <header className="topbar"><div className="brand"><span className={"brand-mark " + (isOnline ? "active" : "")}/><span>PumatVPN</span></div><span className="version">v0.1.0</span></header>
    <section className="content">
      <div className="eyebrow">VPN CONNECTION</div>
      <div className={"status-dot " + status}><span/></div>
      <h1>{statusText}</h1>
      <p className="subtitle">{isOnline ? "sing-box is running" : "Ready to start sing-box"}</p>
      <div className="config-card"><div className="config-icon">{"{}"}</div><div className="config-info"><strong>config.json</strong><span>Configuration file</span></div><div className="config-state"><span className="check">✓</span>Ready</div></div>
      <button className={"connect-button " + (isOnline ? "disconnect" : "")} onClick={toggle} disabled={isConnecting}>{isConnecting ? "Connecting..." : isOnline ? "Disconnect" : "Connect"}</button>
      <div className="engine-status"><span className="engine-label">Sing-box</span><span className="engine-value"><i className="online-indicator"/>Binary ready</span></div>
    </section>
  </main>;
}
