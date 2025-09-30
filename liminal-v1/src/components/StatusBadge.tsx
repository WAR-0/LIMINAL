import { motion } from 'framer-motion';
import { cn } from '../lib/utils';

type StatusType = 'idle' | 'active' | 'working' | 'error' | 'success';

interface StatusBadgeProps {
  status: StatusType;
  label?: string;
  animate?: boolean;
  className?: string;
}

const statusConfig = {
  idle: {
    color: 'text-text-muted',
    bg: 'bg-bg-tertiary',
    border: 'border-border',
    dotColor: 'bg-text-muted',
  },
  active: {
    color: 'text-accent',
    bg: 'bg-accent/10',
    border: 'border-accent/50',
    dotColor: 'bg-accent',
  },
  working: {
    color: 'text-warning',
    bg: 'bg-warning/10',
    border: 'border-warning/50',
    dotColor: 'bg-warning',
  },
  error: {
    color: 'text-error',
    bg: 'bg-error/10',
    border: 'border-error/50',
    dotColor: 'bg-error',
  },
  success: {
    color: 'text-success',
    bg: 'bg-success/10',
    border: 'border-success/50',
    dotColor: 'bg-success',
  },
};

export function StatusBadge({
  status,
  label,
  animate = true,
  className,
}: StatusBadgeProps) {
  const config = statusConfig[status];

  if (!animate) {
    return (
      <div
        className={cn(
          'inline-flex items-center gap-2 px-3 py-1.5 rounded-md border text-sm font-medium',
          config.bg,
          config.color,
          config.border,
          className
        )}
      >
        <span className={cn('w-2 h-2 rounded-full', config.dotColor)} />
        {label || status}
      </div>
    );
  }

  return (
    <motion.div
      className={cn(
        'inline-flex items-center gap-2 px-3 py-1.5 rounded-md border text-sm font-medium',
        config.bg,
        config.color,
        config.border,
        className
      )}
      animate={status !== 'idle' ? {
        scale: [1, 1.05, 1],
        opacity: [0.8, 1, 0.8],
      } : undefined}
      transition={{
        duration: 2,
        repeat: Infinity,
        ease: 'easeInOut' as const,
      }}
    >
      <span className={cn('w-2 h-2 rounded-full', config.dotColor)} />
      {label || status}
    </motion.div>
  );
}