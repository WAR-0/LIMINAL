import { motion } from 'framer-motion';
import { Bot, Zap, Moon, User, Circle } from 'lucide-react';
import { cn } from '../lib/utils';

type AgentRole = 'explorer' | 'builder' | 'analyzer' | 'director' | 'custom';
type AgentStatus = 'idle' | 'thinking' | 'responding';

interface AgentIndicatorProps {
  role: AgentRole;
  status: AgentStatus;
  label?: string;
  className?: string;
  onClick?: () => void;
}

const agentConfig = {
  explorer: {
    icon: Zap,
    color: 'text-agent-explorer',
    bgColor: 'bg-agent-explorer/10',
    glowClass: 'shadow-md shadow-agent-explorer/20',
    label: 'Explorer',
  },
  builder: {
    icon: Bot,
    color: 'text-agent-builder',
    bgColor: 'bg-agent-builder/10',
    glowClass: 'shadow-md shadow-agent-builder/20',
    label: 'Builder',
  },
  analyzer: {
    icon: Moon,
    color: 'text-agent-analyzer',
    bgColor: 'bg-agent-analyzer/10',
    glowClass: 'shadow-md shadow-agent-analyzer/20',
    label: 'Analyzer',
  },
  director: {
    icon: User,
    color: 'text-agent-director',
    bgColor: 'bg-agent-director/10',
    glowClass: 'shadow-md shadow-agent-director/20',
    label: 'Director',
  },
  custom: {
    icon: Circle,
    color: 'text-grey',
    bgColor: 'bg-grey/10',
    glowClass: 'shadow-md shadow-grey/20',
    label: 'Agent',
  },
};

const statusAnimations = {
  idle: {
    scale: 1,
    opacity: 0.6,
  },
  thinking: {
    scale: [1, 1.05, 1],
    opacity: [0.6, 0.8, 0.6],
    transition: {
      duration: 3,
      repeat: Infinity,
      ease: 'easeInOut' as const,
    },
  },
  responding: {
    scale: 1.02,
    opacity: 0.9,
    transition: {
      duration: 0.2,
    },
  },
};

export function AgentIndicator({
  role,
  status,
  label,
  className,
  onClick,
}: AgentIndicatorProps) {
  const config = agentConfig[role];
  const Icon = config.icon;

  return (
    <motion.button
      className={cn(
        'relative p-2 rounded-lg transition-all duration-300',
        config.bgColor,
        config.color,
        'hover:scale-105 cursor-pointer',
        status !== 'idle' && config.glowClass,
        className
      )}
      animate={statusAnimations[status]}
      title={`${label || config.label} - ${status}`}
      onClick={onClick}
    >
      <Icon size={20} />
      {status !== 'idle' && (
        <motion.div
          className={cn('absolute inset-0 rounded-lg', config.bgColor)}
          animate={{
            opacity: [0.1, 0.2, 0.1],
          }}
          transition={{
            duration: 3,
            repeat: Infinity,
            ease: 'easeInOut' as const,
          }}
        />
      )}
    </motion.button>
  );
}