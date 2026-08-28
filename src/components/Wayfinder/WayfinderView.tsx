import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

type Map = { id: string; title: string; status: string; destination: string; frontier_count: number; claimed_count: number; blocked_count: number; fog_count: number; resolved_count: number; updated: string };

export function WayfinderView() {
  const [maps, setMaps] = useState<Map[]>([]); const [error, setError] = useState<string | null>(null);
  useEffect(() => { invoke<{ maps: Map[] }>("get_wayfinder_overview").then((value) => setMaps(value.maps)).catch((reason) => setError(String(reason))); }, []);
  return (
    <main style={{ height: "100%", overflow: "auto", padding: "var(--page-top) var(--page-gutter)" }}>
      <h1>Wayfinder <small style={{ color: "var(--text-tertiary)", fontWeight: 400 }}>Experimental</small></h1>
      <p style={{ color: "var(--text-tertiary)" }}>Decision maps shape uncertain multi-session work before it becomes a specification. This view is read-only.</p>
      {error ? <p role="alert">{error}</p> : null}
      {maps.length === 0 && !error ? <p>No decision maps yet. Use Wayfinder only after explicit operator agreement.</p> : null}
      {maps.map((map) => (
        <article key={map.id} style={{ border: "1px solid var(--border-primary)", borderRadius: 10, padding: 16, marginBottom: 12 }}>
          <strong>{map.title}</strong> <code>{map.id}</code>
          <p>{map.destination}</p>
          <p style={{ color: "var(--text-tertiary)" }}>{map.status} · frontier {map.frontier_count} · claimed {map.claimed_count} · blocked {map.blocked_count} · fog {map.fog_count} · resolved {map.resolved_count}</p>
        </article>
      ))}
    </main>
  );
}
