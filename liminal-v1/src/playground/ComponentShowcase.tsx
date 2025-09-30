import { useState } from 'react';
import {
  Button,
  StatusBadge,
  Panel,
  AgentIndicator,
  Terminal,
} from '../components';
import { colors } from '../theme';

export function ComponentShowcase() {
  const [agentStatus, setAgentStatus] = useState<'idle' | 'thinking' | 'responding'>('idle');

  return (
    <div className="min-h-screen bg-bg-primary text-text-primary p-8 overflow-auto">
      <div className="max-w-7xl mx-auto space-y-8">
        <div>
          <h1 className="text-3xl font-bold text-text-primary mb-2">
            LIMINAL Design System
          </h1>
          <p className="text-text-muted">
            Component library adapted from UNCAN for the LIMINAL V1 MVP
          </p>
        </div>

        <Panel
          title="Color Palette"
          subtitle="Core colors from UNCAN theme"
        >
          <div className="space-y-6">
            <div>
              <h4 className="text-sm font-semibold text-text-primary mb-3">Background</h4>
              <div className="grid grid-cols-4 gap-4">
                {Object.entries(colors.background).map(([name, color]) => (
                  <div key={name} className="space-y-2">
                    <div
                      className="h-16 rounded-lg border border-border"
                      style={{ backgroundColor: color }}
                    />
                    <div className="text-xs">
                      <div className="text-text-primary font-medium">{name}</div>
                      <div className="text-text-muted font-mono">{color}</div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div>
              <h4 className="text-sm font-semibold text-text-primary mb-3">Agent Colors</h4>
              <div className="grid grid-cols-4 gap-4">
                {Object.entries(colors.agent).map(([name, color]) => (
                  <div key={name} className="space-y-2">
                    <div
                      className="h-16 rounded-lg border border-border"
                      style={{ backgroundColor: color }}
                    />
                    <div className="text-xs">
                      <div className="text-text-primary font-medium">{name}</div>
                      <div className="text-text-muted font-mono">{color}</div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div>
              <h4 className="text-sm font-semibold text-text-primary mb-3">Status Colors</h4>
              <div className="grid grid-cols-4 gap-4">
                {Object.entries(colors.status).map(([name, color]) => (
                  <div key={name} className="space-y-2">
                    <div
                      className="h-16 rounded-lg border border-border"
                      style={{ backgroundColor: color }}
                    />
                    <div className="text-xs">
                      <div className="text-text-primary font-medium">{name}</div>
                      <div className="text-text-muted font-mono">{color}</div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </Panel>

        <Panel
          title="Buttons"
          subtitle="Primary, secondary, ghost, and danger variants"
        >
          <div className="space-y-4">
            <div className="flex items-center gap-4">
              <Button variant="primary">Primary</Button>
              <Button variant="secondary">Secondary</Button>
              <Button variant="ghost">Ghost</Button>
              <Button variant="danger">Danger</Button>
            </div>
            <div className="flex items-center gap-4">
              <Button variant="primary" size="sm">Small</Button>
              <Button variant="primary" size="md">Medium</Button>
              <Button variant="primary" size="lg">Large</Button>
            </div>
            <div className="flex items-center gap-4">
              <Button variant="primary" disabled>Disabled</Button>
            </div>
          </div>
        </Panel>

        <Panel
          title="Status Badges"
          subtitle="Visual indicators for system states"
        >
          <div className="flex flex-wrap gap-4">
            <StatusBadge status="idle" label="Idle" />
            <StatusBadge status="active" label="Active" />
            <StatusBadge status="working" label="Working" />
            <StatusBadge status="error" label="Error" />
            <StatusBadge status="success" label="Success" />
          </div>
        </Panel>

        <Panel
          title="Agent Indicators"
          subtitle="Animated agent status indicators"
          headerAction={
            <div className="flex gap-2">
              <Button size="sm" onClick={() => setAgentStatus('idle')}>
                Idle
              </Button>
              <Button size="sm" onClick={() => setAgentStatus('thinking')}>
                Thinking
              </Button>
              <Button size="sm" onClick={() => setAgentStatus('responding')}>
                Responding
              </Button>
            </div>
          }
        >
          <div className="flex gap-4">
            <AgentIndicator role="explorer" status={agentStatus} />
            <AgentIndicator role="builder" status={agentStatus} />
            <AgentIndicator role="analyzer" status={agentStatus} />
            <AgentIndicator role="director" status={agentStatus} />
            <AgentIndicator role="custom" status={agentStatus} label="Custom" />
          </div>
        </Panel>

        <Panel
          title="Panel Variants"
          subtitle="Default, glass, and solid panel styles"
        >
          <div className="grid grid-cols-3 gap-4">
            <Panel variant="default" title="Default">
              <p className="text-text-secondary text-sm">
                Standard panel with solid background
              </p>
            </Panel>
            <Panel variant="glass" title="Glass">
              <p className="text-text-secondary text-sm">
                Translucent panel with backdrop blur
              </p>
            </Panel>
            <Panel variant="solid" title="Solid">
              <p className="text-text-secondary text-sm">
                Opaque panel with tertiary background
              </p>
            </Panel>
          </div>
        </Panel>

        <Panel
          title="Terminal Component"
          subtitle="Terminal-style output display"
        >
          <Terminal
            lines={[
              '\x1b[1;35mWelcome to LIMINAL V1 MVP\x1b[0m',
              '\x1b[90mMulti-Agent Orchestration Platform\x1b[0m',
              '',
              '\x1b[32m✓\x1b[0m Agent A: Ready',
              '\x1b[32m✓\x1b[0m Agent B: Ready',
              '\x1b[33m⚠\x1b[0m Waiting for message...',
            ]}
            className="h-48"
          />
        </Panel>

        <Panel
          title="Typography"
          subtitle="Font sizes and weights"
        >
          <div className="space-y-4">
            <div>
              <div className="text-xs text-text-muted mb-1">Extra Small (xs)</div>
              <div className="text-xs">The quick brown fox jumps over the lazy dog</div>
            </div>
            <div>
              <div className="text-xs text-text-muted mb-1">Small (sm)</div>
              <div className="text-sm">The quick brown fox jumps over the lazy dog</div>
            </div>
            <div>
              <div className="text-xs text-text-muted mb-1">Base</div>
              <div className="text-base">The quick brown fox jumps over the lazy dog</div>
            </div>
            <div>
              <div className="text-xs text-text-muted mb-1">Large (lg)</div>
              <div className="text-lg">The quick brown fox jumps over the lazy dog</div>
            </div>
            <div>
              <div className="text-xs text-text-muted mb-1">Extra Large (xl)</div>
              <div className="text-xl">The quick brown fox jumps over the lazy dog</div>
            </div>
          </div>
        </Panel>
      </div>
    </div>
  );
}