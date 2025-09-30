import { HTMLAttributes, ReactNode } from 'react';
import { cn } from '../lib/utils';

interface PanelProps extends HTMLAttributes<HTMLDivElement> {
  title?: string;
  subtitle?: string;
  children: ReactNode;
  variant?: 'default' | 'glass' | 'solid';
  headerAction?: ReactNode;
}

export function Panel({
  title,
  subtitle,
  children,
  variant = 'default',
  headerAction,
  className,
  ...props
}: PanelProps) {
  const variants = {
    default: 'bg-bg-secondary border-border',
    glass: 'bg-bg-secondary/80 backdrop-blur-sm border-border/50',
    solid: 'bg-bg-tertiary border-border',
  };

  return (
    <div
      className={cn(
        'rounded-lg border overflow-hidden',
        variants[variant],
        className
      )}
      {...props}
    >
      {(title || subtitle || headerAction) && (
        <div className="px-4 py-3 border-b border-border flex items-center justify-between">
          <div>
            {title && (
              <h3 className="text-text-primary font-semibold text-base">
                {title}
              </h3>
            )}
            {subtitle && (
              <p className="text-text-muted text-sm mt-0.5">
                {subtitle}
              </p>
            )}
          </div>
          {headerAction && (
            <div className="flex items-center gap-2">
              {headerAction}
            </div>
          )}
        </div>
      )}
      <div className="p-4">
        {children}
      </div>
    </div>
  );
}