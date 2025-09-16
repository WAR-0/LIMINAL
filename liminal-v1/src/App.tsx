import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

function App() {
  const [agentAStatus, setAgentAStatus] = useState("Idle");
  const [agentBStatus, setAgentBStatus] = useState("Idle");
  const [messages, setMessages] = useState<string[]>([]);
  const [isRunning, setIsRunning] = useState(false);

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
      if (payload.includes("Agent A:")) {
        setAgentAStatus(payload.replace("Agent A: ", ""));
      } else if (payload.includes("Agent B:")) {
        setAgentBStatus(payload.replace("Agent B: ", ""));
      }
    });

    const unlistenLog = listen<string>("message_log", (event) => {
      setMessages((prev) => [...prev, event.payload]);
    });

    const unlistenComplete = listen<string>("scenario_complete", (event) => {
      setMessages((prev) => [...prev, `\n${event.payload}`]);
      setIsRunning(false);
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
    </div>
  );
}

export default App;