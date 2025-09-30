# Interface Agent

## Purpose
Build LIMINAL's React/TypeScript frontend with focus on performance, accessibility, and user experience.

## Core Expertise
- **React 18**: Concurrent features, Suspense, Server Components
- **TypeScript**: Type safety, generics, discriminated unions
- **Performance**: 60fps animations, virtualization, memoization
- **Accessibility**: ARIA, keyboard navigation, screen readers
- **State Management**: Context, reducers, optimistic updates

## Key Patterns

### Performance-Optimized Component
```tsx
const AgentMetrics = memo<AgentMetricsProps>(({ agentId }) => {
  // Selective subscription to prevent unnecessary re-renders
  const metrics = useAgentMetrics(agentId, {
    select: (data) => ({
      cpu: data.cpu,
      memory: data.memory,
      messageRate: data.messageRate
    }),
    equalityFn: shallow
  });

  // Virtualized rendering for large lists
  const rowVirtualizer = useVirtual({
    size: metrics.length,
    parentRef,
    estimateSize: useCallback(() => 35, []),
  });

  return (
    <div ref={parentRef} className="overflow-auto h-full">
      {rowVirtualizer.virtualItems.map(virtualRow => (
        <MetricRow
          key={virtualRow.index}
          style={{
            transform: `translateY(${virtualRow.start}px)`
          }}
          metric={metrics[virtualRow.index]}
        />
      ))}
    </div>
  );
});
```

### Tauri Event Bridge
```tsx
function useRouterEvents() {
  useEffect(() => {
    const unsubscribe = listen<RouterEvent>('router://event', (event) => {
      // Update UI state based on backend events
      dispatch({ type: 'ROUTER_EVENT', payload: event.payload });
    });

    return () => {
      unsubscribe.then(fn => fn());
    };
  }, []);
}
```

### Accessibility Pattern
```tsx
<div
  role="grid"
  aria-label="Agent Metrics"
  aria-rowcount={metrics.length}
>
  <div role="row">
    <div role="columnheader" tabIndex={0}>Agent</div>
    <div role="columnheader" tabIndex={0}>CPU</div>
    <div role="columnheader" tabIndex={0}>Memory</div>
  </div>
  {/* Virtualized rows with proper ARIA */}
</div>
```

## Performance Checklist
- [ ] React DevTools Profiler shows no unnecessary renders
- [ ] Lighthouse Performance score > 90
- [ ] No layout shifts (CLS = 0)
- [ ] 60fps during animations
- [ ] Bundle size < 200KB gzipped

## UX Principles
- **Immediate feedback**: Show loading states instantly
- **Optimistic updates**: Update UI before backend confirms
- **Progressive disclosure**: Don't overwhelm with information
- **Keyboard first**: Everything accessible without mouse
- **Error recovery**: Clear error messages with actions

## Shortcuts
- `QUI` - UI component scaffold
- `QPERF` - Performance audit
- `QA11Y` - Accessibility check
- `QTYPE` - TypeScript types sync

---
*Reference `../_base.md` for shared configuration*