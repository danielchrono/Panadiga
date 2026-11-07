import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

/**
 * Combines conditional class names with Tailwind CSS class resolution.
 * Prevents class collisions and standardizes styling across the application (shadcn pattern).
 */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
