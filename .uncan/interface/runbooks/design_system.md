# LIMINAL Design System

**Status**: Implemented
**Based on**: UNCAN UI Theme
**Date**: 2025-09-29

## Overview

The LIMINAL design system is adapted from the UNCAN project, featuring a dark terminal aesthetic with high contrast colors, subtle animations, and a focus on readability. The system is built with Tailwind CSS and uses JetBrains Mono as the primary monospace font.

## Color Palette

### Background Colors

| Name | Hex | Usage |
|------|-----|-------|
| `bg-primary` | `#0a0e1a` | Main application background |
| `bg-secondary` | `#0f1726` | Panel and card backgrounds |
| `bg-tertiary` | `#1a2332` | Hover states, secondary panels |
| `bg-chat` | `#0d1420` | Chat/message backgrounds |

### Text Colors

| Name | Hex | Usage |
|------|-----|-------|
| `text-primary` | `#e8eaed` | Primary text, headings |
| `text-secondary` | `#9ca3af` | Secondary text, labels |
| `text-muted` | `#6b7280` | Disabled/muted text |

### Border Colors

| Name | Hex | Usage |
|------|-----|-------|
| `border` | `#1f2937` | Standard borders |
| `border-subtle` | `#161e2e` | Subtle dividers |

### Agent Role Colors

| Role | Name | Hex | Visual |
|------|------|-----|--------|
| Explorer | `agent-explorer` | `#89dceb` | Cyan |
| Builder | `agent-builder` | `#a6e3a1` | Green |
| Analyzer | `agent-analyzer` | `#cba6f7` | Purple |
| Director | `agent-director` | `#f9e2af` | Yellow |

### Status Colors

| Status | Name | Hex | Usage |
|--------|------|-----|-------|
| Accent | `accent` | `#cba6f7` | Primary actions, highlights |
| Success | `success` | `#a6e3a1` | Success states |
| Warning | `warning` | `#f9e2af` | Warning states |
| Error | `error` | `#f38ba8` | Error states |

### Terminal Colors (Catppuccin-inspired)

Full terminal color scheme is defined in `src/theme/colors.ts` with 16 ANSI colors for rich terminal output.

## Typography

### Font Family

- **Monospace**: `JetBrains Mono`, `Fira Code`, `Courier New`, `monospace`

### Font Sizes

| Size | Value | Usage |
|------|-------|-------|
| `text-xs` | `0.75rem` | Small labels, captions |
| `text-sm` | `0.875rem` | Body text, buttons |
| `text-base` | `1rem` | Default body text |
| `text-lg` | `1.125rem` | Subheadings |
| `text-xl` | `1.25rem` | Headings |
| `text-2xl` | `1.5rem` | Large headings |
| `text-3xl` | `1.875rem` | Page titles |

## Components

### Button

Variants: `primary`, `secondary`, `ghost`, `danger`
Sizes: `sm`, `md`, `lg`

```tsx
import { Button } from '@/components';

<Button variant="primary" size="md">
  Click me
</Button>
```

**Design Guidelines**:
- Use `primary` for main actions
- Use `secondary` for secondary actions
- Use `ghost` for tertiary/subtle actions
- Use `danger` for destructive actions
- Disabled state automatically applied via `disabled` prop

### StatusBadge

Visual indicator for system states with optional animation.

```tsx
import { StatusBadge } from '@/components';

<StatusBadge status="active" label="Agent Active" animate />
```

**Available Statuses**: `idle`, `active`, `working`, `error`, `success`

### Panel

Container component with optional title, subtitle, and header actions.

```tsx
import { Panel } from '@/components';

<Panel
  title="System Metrics"
  subtitle="Real-time performance data"
  variant="glass"
  headerAction={<Button size="sm">Refresh</Button>}
>
  {/* Content */}
</Panel>
```

**Variants**:
- `default`: Solid background with border
- `glass`: Translucent with backdrop blur
- `solid`: Opaque tertiary background

### AgentIndicator

Animated indicator for agent status with role-based styling.

```tsx
import { AgentIndicator } from '@/components';

<AgentIndicator
  role="explorer"
  status="thinking"
  label="Explorer Agent"
/>
```

**Roles**: `explorer`, `builder`, `analyzer`, `director`, `custom`
**Statuses**: `idle`, `thinking`, `responding`

### Terminal

Terminal-style output display with ANSI color support.

```tsx
import { Terminal } from '@/components';

<Terminal
  lines={[
    'Welcome to LIMINAL',
    'Agent A: Ready',
    'Agent B: Ready'
  ]}
  prompt="$"
/>
```

## Animations

### Breathing Animation

Pulsing effect for active states (4s duration).

```tsx
className="animate-breathing"
```

### Flash Animation

Quick flash effect (0.5s duration).

```tsx
className="animate-flash"
```

### Pulse Slow

Slow pulse for loading states (3s duration).

```tsx
className="animate-pulse-slow"
```

## Visual Effects

### Glass Effect

Translucent background with backdrop blur.

```tsx
className="glass-effect"
```

### Agent Glow

Subtle shadow effects for agent indicators (automatically applied by `AgentIndicator` component).

## Accessibility

- All interactive elements have focus states (`focus:ring-2`)
- Proper ARIA labels should be added to buttons and interactive elements
- Color contrast meets WCAG AA standards
- Keyboard navigation supported on all interactive components

## Usage Guidelines

### Color Usage

- **Background hierarchy**: Use `bg-primary` → `bg-secondary` → `bg-tertiary` for depth
- **Text hierarchy**: Use `text-primary` → `text-secondary` → `text-muted` for importance
- **Agent colors**: Reserve for agent-specific UI elements (indicators, badges, etc.)
- **Status colors**: Use consistently for success/warning/error states

### Spacing

Follow Tailwind's spacing scale:
- `gap-2`, `gap-4` for component spacing
- `p-4` for standard panel padding
- `px-3 py-2` for button padding

### Borders

- Use `border-border` for visible borders
- Use `border-border-subtle` for subtle dividers
- Use `rounded-lg` for standard border radius

## Component Playground

Access the component playground at `#/playground` to see all components and colors in action.

Navigate to: `http://localhost:1420/#/playground` (dev) or use hash navigation in the app.

## Files

- `src/theme/colors.ts` - Color constants
- `src/theme/styles.ts` - Style constants (spacing, shadows, typography)
- `src/theme/index.ts` - Theme exports
- `src/components/` - Component library
- `src/lib/utils.ts` - Utility functions (cn helper)
- `src/index.css` - Global styles and Tailwind directives
- `tailwind.config.js` - Tailwind configuration

## Future Enhancements

- Add more component variants (tabs, modals, tooltips)
- Implement theme switching (light mode option)
- Add more animation presets
- Create icon library integration
- Add responsive breakpoint documentation