import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

type SerializedSystemTime = {
  secs_since_epoch: number;
  nanos_since_epoch: number;
};

interface PerformanceMetrics {
  messageRoutingLatencyMs: number;
  agentSpawnTimeMs: number;
  leaseAcquisitionTimeMs: number;
  totalMessagesRouted: number;
  totalLeasesAcquired: number;
  memoryUsageMb: number;
  rateLimitedMessages: number;
}

interface RouterSnapshot {
  queueDepths: Record<string, number>;
  lastDispatchedPriority?: string | null;
  lastDispatchedAt?: SerializedSystemTime | null;
  rateLimitedMessages: number;
}

interface RateLimitSnapshot {
  sender: string;
  tokensRemaining: number;
  capacity: number;
  refillRate: number;
  lastRefill?: SerializedSystemTime | null;
  rateLimitHits: number;
}

interface LeaseSnapshotSummary {
  activeLeases: number;
  totalPending: number;
  pendingByResource: Record<string, number>;
  deferrals: number;
  overrides: number;
  escalations: number;
  outstandingLeaseIds: number[];
}

interface PtyLastEvent {
  agentId: string;
  eventName?: string | null;
  timestamp: SerializedSystemTime;
}

interface PtySnapshot {
  eventsByName: Record<string, number>;
  totalEvents: number;
  lastEvent?: PtyLastEvent | null;
}

interface SystemSnapshot {
  memoryUsageMb: number;
  lastUpdated?: SerializedSystemTime | null;
}

interface MetricsSnapshot {
  performance: PerformanceMetrics;
  router: RouterSnapshot;
  rateLimits: RateLimitSnapshot[];
  leases: LeaseSnapshotSummary;
  pty: PtySnapshot;
  system: SystemSnapshot;
}

interface HealthAlert {
  severity: string;
  message: string;
  context: Record<string, unknown>;
}

const PRIORITY_ORDER = [
  "info",
  "coordinate",
  "blocking",
  "critical",
  "directorOverride",
];

const formatSystemTime = (value?: SerializedSystemTime | null) => {
  if (!value) {
    return "—";
  }
  const millis = value.secs_since_epoch * 1000 + Math.floor(value.nanos_since_epoch / 1_000_000);
  return new Date(millis).toLocaleTimeString();
};

const formatLabel = (label: string) => {
  if (!label) {
    return label;
  }
  const withSpaces = label.replace(/([A-Z])/g, " $1");
  return withSpaces.charAt(0).toUpperCase() + withSpaces.slice(1);
};

const formatNumber = (value: number, fraction = 2) => {
  if (!Number.isFinite(value)) {
    return "—";
  }
  return value.toFixed(fraction);
};

function App() {
  const [agentAStatus, setAgentAStatus] = useState("Idle");
  const [agentBStatus, setAgentBStatus] = useState("Idle");
  const [messages, setMessages] = useState<string[]>([]);
  const [isScenarioRunning, setIsScenarioRunning] = useState(false);
  const [metricsSnapshot, setMetricsSnapshot] = useState<MetricsSnapshot | null>(null);
  const [isStreaming, setIsStreaming] = useState(false);
  const [isRefreshingMetrics, setIsRefreshingMetrics] = useState(false);
  const [metricsError, setMetricsError] = useState<string | null>(null);

  const refreshMetrics = useCallback(async () => {
    setIsRefreshingMetrics(true);
    try {
      const snapshot = await invoke<MetricsSnapshot>("get_metrics_snapshot");
      setMetricsSnapshot(snapshot);
      setMetricsError(null);
    } catch (error) {
      console.error("Unable to fetch metrics snapshot", error);
      setMetricsError("Unable to fetch metrics snapshot");
    } finally {
      setIsRefreshingMetrics(false);
    }
  }, []);

  useEffect(() => {
    let cancelled = false;

    const bootstrap = async () => {
      try {
        await invoke("start_metrics_stream");
        if (!cancelled) {
          setIsStreaming(true);
        }
      } catch (error) {
        console.warn("Metrics stream not available", error);
      }
      if (!cancelled) {
        await refreshMetrics();
      }
    };

    bootstrap();

    const unlistenMetrics = listen<MetricsSnapshot>("metrics_snapshot", (event) => {
      setMetricsSnapshot(event.payload);
    });

    const unlistenHealth = listen<HealthAlert>("health_alert", (event) => {
      const alert = event.payload;
      setMessages((prev) => [
        ...prev,
        `[${alert.severity.toUpperCase()}] ${alert.message}`,
      ]);
    });

    const unlistenStatus = listen<string>("agent_status", (event) => {
      const payload = event.payload;
      if (payload.includes("Agent A:")) {
        setAgentAStatus(payload.replace("Agent A: ", ""));
      } else if (payload.includes("Agent A (PTY):")) {
        setAgentAStatus(payload.replace("Agent A (PTY): ", ""));
      } else if (payload.includes("Agent B:")) {
        setAgentBStatus(payload.replace("Agent B: ", ""));
      } else if (payload.includes("Agent B (PTY):")) {
        setAgentBStatus(payload.replace("Agent B (PTY): ", ""));
      } else if (payload.includes("Spawned")) {
        setAgentAStatus("PTY Process Spawned");
        setAgentBStatus("PTY Process Spawned");
      }
    });

    const unlistenLog = listen<string>("message_log", (event) => {
      setMessages((prev) => [...prev, event.payload]);
    });

    const unlistenComplete = listen<string>("scenario_complete", async (event) => {
      setMessages((prev) => [...prev, `\n${event.payload}`]);
      setIsScenarioRunning(false);
      await refreshMetrics();
    });

    return () => {
      cancelled = true;
      unlistenMetrics.then((unsubscribe) => unsubscribe());
      unlistenStatus.then((unsubscribe) => unsubscribe());
      unlistenLog.then((unsubscribe) => unsubscribe());
      unlistenComplete.then((unsubscribe) => unsubscribe());
      unlistenHealth.then((unsubscribe) => unsubscribe());
    };
  }, [refreshMetrics]);

  useEffect(() => {
    const initialiseAgentStatus = async () => {
      const statusA = await invoke<string>("get_agent_status", { agentId: "Agent_A" });
      const statusB = await invoke<string>("get_agent_status", { agentId: "Agent_B" });
      setAgentAStatus(statusA);
      setAgentBStatus(statusB);
    };

    initialiseAgentStatus();
  }, []);

  const handleStartScenario = async () => {
    setIsScenarioRunning(true);
    setMessages(["Starting scenario..."]);
    setAgentAStatus("Idle");
    setAgentBStatus("Idle");

    try {
      await invoke("start_scenario");
    } catch (error) {
      console.error("Error starting scenario", error);
      setMessages((prev) => [...prev, `Error: ${error}`]);
      setIsScenarioRunning(false);
    }
  };

  const handleStartPtyScenario = async () => {
    setIsScenarioRunning(true);
    setMessages(["Starting PTY scenario with real processes..."]);
    setAgentAStatus("Spawning...");
    setAgentBStatus("Spawning...");

    try {
      await invoke("start_pty_scenario");
    } catch (error) {
      console.error("Error starting PTY scenario", error);
      setMessages((prev) => [...prev, `Error: ${error}`]);
      setIsScenarioRunning(false);
    }
  };

  const handleSimulateRouterLoad = async () => {
    try {
      await invoke("simulate_router_load");
      setMessages((prev) => [...prev, "Triggered synthetic router load"]);
      await refreshMetrics();
    } catch (error) {
      console.error("Failed to simulate router load", error);
      setMessages((prev) => [...prev, `Router load error: ${error}`]);
    }
  };

  const handleSimulateLeaseContention = async () => {
    try {
      await invoke("simulate_lease_contention");
      setMessages((prev) => [...prev, "Triggered synthetic lease contention"]);
      await refreshMetrics();
    } catch (error) {
      console.error("Failed to simulate lease contention", error);
      setMessages((prev) => [...prev, `Lease contention error: ${error}`]);
    }
  };

  const queueDepthEntries = PRIORITY_ORDER.map((priority) => ({
    priority,
    depth: metricsSnapshot?.router.queueDepths[priority] ?? 0,
  }));

  const tokenBuckets = metricsSnapshot?.rateLimits ?? [];
  const leaseSummary = metricsSnapshot?.leases;
  const ptySummary = metricsSnapshot?.pty;
  const performance = metricsSnapshot?.performance;
  const system = metricsSnapshot?.system;

  return (
    <div className="container">
      <h1>LIMINAL V1 MVP</h1>

      <div className="control-panel">
        <button
          onClick={handleStartScenario}
          className="start-button"
          disabled={isScenarioRunning}
        >
          {isScenarioRunning ? "Running..." : "Start Scenario"}
        </button>
        <button
          onClick={handleStartPtyScenario}
          className="start-button pty-button"
          disabled={isScenarioRunning}
        >
          {isScenarioRunning ? "Running..." : "Start PTY Scenario"}
        </button>
      </div>

      <div className="status-panel">
        <h2>Agent Status</h2>
        <div className="agent-status">
          <p><strong>Agent A:</strong> {agentAStatus}</p>
          <p><strong>Agent B:</strong> {agentBStatus}</p>
        </div>
      </div>

      <div className="message-log">
        <h2>Message Log</h2>
        <pre className="log-content">
          {messages.length > 0 ? messages.join("\n") : "No messages yet..."}
        </pre>
      </div>

      <div className="metrics-dashboard">
        <div className="metrics-top-bar">
          <h2>Metrics & Observability</h2>
          <div className="metrics-actions">
            <span className={`stream-chip ${isStreaming ? "streaming" : "offline"}`}>
              {isStreaming ? "Streaming updates" : "Manual refresh"}
            </span>
            <button
              className="secondary-button"
              onClick={refreshMetrics}
              disabled={isRefreshingMetrics}
            >
              {isRefreshingMetrics ? "Refreshing..." : "Refresh Snapshot"}
            </button>
            <button className="secondary-button" onClick={handleSimulateRouterLoad}>
              Simulate Router Load
            </button>
            <button className="secondary-button" onClick={handleSimulateLeaseContention}>
              Simulate Lease Contention
            </button>
          </div>
        </div>

        {metricsError && <div className="metrics-error">{metricsError}</div>}

        {metricsSnapshot ? (
          <div className="metrics-grid">
            <div className="metrics-card">
              <h3>Router</h3>
              <div className="metric-row">
                <span>Last priority</span>
                <strong>
                  {metricsSnapshot.router.lastDispatchedPriority
                    ? formatLabel(metricsSnapshot.router.lastDispatchedPriority)
                    : "—"}
                </strong>
              </div>
              <div className="metric-row">
                <span>Last dispatch</span>
                <strong>{formatSystemTime(metricsSnapshot.router.lastDispatchedAt)}</strong>
              </div>
              <div className="metric-row">
                <span>Rate-limited messages</span>
                <strong>{metricsSnapshot.router.rateLimitedMessages}</strong>
              </div>
              <table className="metrics-table">
                <thead>
                  <tr>
                    <th>Priority</th>
                    <th>Queue Depth</th>
                  </tr>
                </thead>
                <tbody>
                  {queueDepthEntries.map(({ priority, depth }) => (
                    <tr key={priority}>
                      <td>{formatLabel(priority)}</td>
                      <td>{depth}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            <div className="metrics-card">
              <h3>Token Buckets</h3>
              {tokenBuckets.length === 0 ? (
                <p className="muted">No token buckets initialised.</p>
              ) : (
                <table className="metrics-table">
                  <thead>
                    <tr>
                      <th>Sender</th>
                      <th>Tokens</th>
                      <th>Capacity</th>
                      <th>Hits</th>
                      <th>Last Refill</th>
                    </tr>
                  </thead>
                  <tbody>
                    {tokenBuckets.map((bucket) => (
                      <tr key={bucket.sender}>
                        <td>{bucket.sender}</td>
                        <td>{formatNumber(bucket.tokensRemaining, 1)}</td>
                        <td>{formatNumber(bucket.capacity, 1)}</td>
                        <td>{bucket.rateLimitHits}</td>
                        <td>{formatSystemTime(bucket.lastRefill)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>

            <div className="metrics-card">
              <h3>Leases</h3>
              {leaseSummary ? (
                <>
                  <div className="metric-row">
                    <span>Active leases</span>
                    <strong>{leaseSummary.activeLeases}</strong>
                  </div>
                  <div className="metric-row">
                    <span>Total pending</span>
                    <strong>{leaseSummary.totalPending}</strong>
                  </div>
                  <div className="metric-row">
                    <span>Deferrals</span>
                    <strong>{leaseSummary.deferrals}</strong>
                  </div>
                  <div className="metric-row">
                    <span>Overrides</span>
                    <strong>{leaseSummary.overrides}</strong>
                  </div>
                  <div className="metric-row">
                    <span>Escalations</span>
                    <strong>{leaseSummary.escalations}</strong>
                  </div>
                  <div className="subsection">
                    <strong>Pending by resource</strong>
                    {Object.keys(leaseSummary.pendingByResource).length === 0 ? (
                      <p className="muted">No queued requests.</p>
                    ) : (
                      <ul>
                        {Object.entries(leaseSummary.pendingByResource).map(([resource, depth]) => (
                          <li key={resource}>
                            <span>{resource}</span>
                            <span className="badge">{depth}</span>
                          </li>
                        ))}
                      </ul>
                    )}
                  </div>
                  <div className="subsection">
                    <strong>Outstanding Lease IDs</strong>
                    {leaseSummary.outstandingLeaseIds.length === 0 ? (
                      <p className="muted">None</p>
                    ) : (
                      <p className="lease-ids">{leaseSummary.outstandingLeaseIds.join(", ")}</p>
                    )}
                  </div>
                </>
              ) : (
                <p className="muted">Lease metrics unavailable.</p>
              )}
            </div>

            <div className="metrics-card">
              <h3>PTY Events</h3>
              {ptySummary ? (
                <>
                  <div className="metric-row">
                    <span>Total events</span>
                    <strong>{ptySummary.totalEvents}</strong>
                  </div>
                  <div className="subsection">
                    <strong>Events by name</strong>
                    {Object.keys(ptySummary.eventsByName).length === 0 ? (
                      <p className="muted">No structured events received.</p>
                    ) : (
                      <ul>
                        {Object.entries(ptySummary.eventsByName).map(([name, count]) => (
                          <li key={name}>
                            <span>{name}</span>
                            <span className="badge">{count}</span>
                          </li>
                        ))}
                      </ul>
                    )}
                  </div>
                  <div className="subsection">
                    <strong>Last event</strong>
                    {ptySummary.lastEvent ? (
                      <p>
                        <span>{ptySummary.lastEvent.eventName ?? "unknown"}</span>
                        <span className="last-event-agent"> from {ptySummary.lastEvent.agentId}</span>
                        <br />
                        <span className="muted">{formatSystemTime(ptySummary.lastEvent.timestamp)}</span>
                      </p>
                    ) : (
                      <p className="muted">No events yet.</p>
                    )}
                  </div>
                </>
              ) : (
                <p className="muted">PTY metrics unavailable.</p>
              )}
            </div>

            <div className="metrics-card">
              <h3>Performance</h3>
              {performance ? (
                <div className="performance-grid">
                  <div>
                    <span>Avg routing (ms)</span>
                    <strong>{formatNumber(performance.messageRoutingLatencyMs)}</strong>
                  </div>
                  <div>
                    <span>Avg lease acquisition (ms)</span>
                    <strong>{formatNumber(performance.leaseAcquisitionTimeMs)}</strong>
                  </div>
                  <div>
                    <span>Agent spawn (ms)</span>
                    <strong>{formatNumber(performance.agentSpawnTimeMs)}</strong>
                  </div>
                  <div>
                    <span>Total messages</span>
                    <strong>{performance.totalMessagesRouted}</strong>
                  </div>
                  <div>
                    <span>Total leases</span>
                    <strong>{performance.totalLeasesAcquired}</strong>
                  </div>
                  <div>
                    <span>Rate-limited</span>
                    <strong>{performance.rateLimitedMessages}</strong>
                  </div>
                  <div>
                    <span>Memory (MB)</span>
                    <strong>{formatNumber(performance.memoryUsageMb)}</strong>
                  </div>
                  <div>
                    <span>Last sample</span>
                    <strong>{formatSystemTime(system?.lastUpdated)}</strong>
                  </div>
                </div>
              ) : (
                <p className="muted">Performance metrics unavailable.</p>
              )}
            </div>
          </div>
        ) : (
          <p className="muted">Metrics have not loaded yet.</p>
        )}
      </div>
    </div>
  );
}

export default App;
