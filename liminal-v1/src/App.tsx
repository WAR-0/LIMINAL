import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

function App() {
  const [agentAStatus, setAgentAStatus] = useState("Idle");
  const [agentBStatus, setAgentBStatus] = useState("Idle");
  const [messages, setMessages] = useState<string[]>([]);

  useEffect(() => {
    const updateAgentStatus = async () => {
      const statusA = await invoke<string>("get_agent_status", { agentId: "Agent_A" });
      const statusB = await invoke<string>("get_agent_status", { agentId: "Agent_B" });
      setAgentAStatus(statusA);
      setAgentBStatus(statusB);
    };

    updateAgentStatus();
  }, []);

  const handleStartScenario = () => {
    invoke("start_scenario");
    setMessages(prev => [...prev, "Scenario started..."]);
  };

  return (
    <div className="container">
      <h1>LIMINAL V1 MVP</h1>

      <div className="control-panel">
        <button onClick={handleStartScenario} className="start-button">
          Start Scenario
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