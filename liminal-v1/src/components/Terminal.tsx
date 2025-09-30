import { HTMLAttributes } from 'react';
import { cn } from '../lib/utils';

interface TerminalProps extends HTMLAttributes<HTMLDivElement> {
  lines?: string[];
  prompt?: string;
}

export function Terminal({
  lines = [],
  prompt = '$',
  className,
  ...props
}: TerminalProps) {
  return (
    <div
      className={cn(
        'bg-[#1e1e2e] text-[#cdd6f4] font-mono text-sm rounded-lg border border-border p-4 overflow-auto',
        className
      )}
      {...props}
    >
      {lines.map((line, index) => (
        <div key={index} className="whitespace-pre-wrap">
          {line}
        </div>
      ))}
      <div className="flex items-center gap-2 mt-2">
        <span className="text-accent">{prompt}</span>
        <span className="animate-pulse">_</span>
      </div>
    </div>
  );
}