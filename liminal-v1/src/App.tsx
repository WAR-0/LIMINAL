import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

interface PerformanceMetrics {
  message_routing_latency_ms: number;
  agent_spawn_time_ms: number;
  lease_acquisition_time_ms: number;
  total_messages_routed: number;
  total_leases_acquired: number;
  memory_usage_mb: number;
}

function App() {
  const [agentAStatus, setAgentAStatus] = useState("Idle");
  const [agentBStatus, setAgentBStatus] = useState("Idle");
  const [messages, setMessages] = useState<string[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const [metrics, setMetrics] = useState<PerformanceMetrics | null>(null);

  useEffect(() => {
    const updateAgentStatus = async () => {
      const statusA = await invoke<string>("get_agent_status", { agentId: "Agent_A" });
      const statusB = await invoke<string>("get_agent_status", { agentId: "Agent_B" });
      setAgentAStatus(statusA);
      setAgentBStatus(statusB);
    };

    updateAgentStatus();

    // Set up event listeners
    const unlistenStatus = listen<string>("agent_status", (event) => {
      const payload = event.payload;
      if (payload.includes("Agent A:") || payload.includes("Agent A (PTY):")) {
        setAgentAStatus(payload.replace("Agent A: ", "").replace("Agent A (PTY): ", ""));
      } else if (payload.includes("Agent B:") || payload.includes("Agent B (PTY):")) {
        setAgentBStatus(payload.replace("Agent B: ", "").replace("Agent B (PTY): ", ""));
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
      setIsRunning(false);
      // Fetch performance metrics after scenario completes
      const perfMetrics = await invoke<PerformanceMetrics>("get_performance_metrics");
      setMetrics(perfMetrics);
    });

    // Cleanup listeners on unmount
    return () => {
      unlistenStatus.then((f) => f());
      unlistenLog.then((f) => f());
      unlistenComplete.then((f) => f());
    };
  }, []);

  const handleStartScenario = async () => {
    setIsRunning(true);
    setMessages(["Starting scenario..."]);
    setAgentAStatus("Idle");
    setAgentBStatus("Idle");

    try {
      await invoke("start_scenario");
    } catch (error) {
      console.error("Error starting scenario:", error);
      setMessages((prev) => [...prev, `Error: ${error}`]);
      setIsRunning(false);
    }
  };

  const handleStartPtyScenario = async () => {
    setIsRunning(true);
    setMessages(["Starting PTY scenario with real processes..."]);
    setAgentAStatus("Spawning...");
    setAgentBStatus("Spawning...");

    try {
      await invoke("start_pty_scenario");
    } catch (error) {
      console.error("Error starting PTY scenario:", error);
      setMessages((prev) => [...prev, `Error: ${error}`]);
      setIsRunning(false);
    }
  };

  return (
    <div className="container">
      <h1>LIMINAL V1 MVP</h1>

      <div className="control-panel">
        <button
          onClick={handleStartScenario}
          className="start-button"
          disabled={isRunning}
        >
          {isRunning ? "Running..." : "Start Scenario"}
        </button>
        <button
          onClick={handleStartPtyScenario}
          className="start-button pty-button"
          disabled={isRunning}
        >
          {isRunning ? "Running..." : "Start PTY Scenario"}
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
          {messages.length > 0 ? messages.join('\n') : 'No messages yet...'}
        </pre>
      </div>

      {metrics && (
        <div className="metrics-panel">
          <h2>Performance Metrics</h2>
          <div className="metrics-grid">
            <div className="metric-item">
              <strong>Message Routing:</strong> {metrics.message_routing_latency_ms.toFixed(2)}ms
            </div>
            <div className="metric-item">
              <strong>Agent Spawn:</strong> {metrics.agent_spawn_time_ms.toFixed(2)}ms
            </div>
            <div className="metric-item">
              <strong>Lease Acquisition:</strong> {metrics.lease_acquisition_time_ms.toFixed(2)}ms
            </div>
            <div className="metric-item">
              <strong>Total Messages:</strong> {metrics.total_messages_routed}
            </div>
            <div className="metric-item">
              <strong>Total Leases:</strong> {metrics.total_leases_acquired}
            </div>
            <div className="metric-item">
              <strong>Memory Usage:</strong> {metrics.memory_usage_mb.toFixed(2)}MB
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;