import React from 'react';

export type StatusVariant = 'success' | 'warning' | 'error' | 'info' | 'neutral';

interface StatusBadgeProps {
  variant: StatusVariant;
  label: string;
  dot?: boolean;
  className?: string;
}

export const StatusBadge: React.FC<StatusBadgeProps> = ({
  variant,
  label,
  dot = true,
  className = '',
}) => {
  const variantStyles: Record<StatusVariant, { bg: string; text: string; dotColor: string; border: string }> = {
    success: {
      bg: 'bg-emerald-500/10',
      text: 'text-emerald-400',
      dotColor: 'bg-emerald-400 animate-pulse',
      border: 'border-emerald-500/30',
    },
    warning: {
      bg: 'bg-amber-500/10',
      text: 'text-amber-400',
      dotColor: 'bg-amber-400',
      border: 'border-amber-500/30',
    },
    error: {
      bg: 'bg-rose-500/10',
      text: 'text-rose-400',
      dotColor: 'bg-rose-400',
      border: 'border-rose-500/30',
    },
    info: {
      bg: 'bg-cyan-500/10',
      text: 'text-cyan-400',
      dotColor: 'bg-cyan-400',
      border: 'border-cyan-500/30',
    },
    neutral: {
      bg: 'bg-slate-500/10',
      text: 'text-slate-400',
      dotColor: 'bg-slate-400',
      border: 'border-slate-500/30',
    },
  };

  const style = variantStyles[variant];

  return (
    <span
      className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-mono font-medium border ${style.bg} ${style.text} ${style.border} ${className}`}
    >
      {dot && <span className={`w-1.5 h-1.5 rounded-full ${style.dotColor}`} />}
      {label}
    </span>
  );
};
