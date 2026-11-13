import * as React from 'react';
import { cn } from '@/lib/utils';

export interface ScrollAreaProps extends React.HTMLAttributes<HTMLDivElement> {
  maxHeightClassName?: string;
}

const ScrollArea = React.forwardRef<HTMLDivElement, ScrollAreaProps>(
  ({ className, maxHeightClassName = 'max-h-96', children, ...props }, ref) => (
    <div
      ref={ref}
      className={cn(
        'overflow-auto pr-1 scrollbar-thin scrollbar-thumb-slate-800',
        maxHeightClassName,
        className
      )}
      {...props}
    >
      {children}
    </div>
  )
);
ScrollArea.displayName = 'ScrollArea';

export { ScrollArea };
