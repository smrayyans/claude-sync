import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import { formatDistanceToNow, parseISO } from "date-fns";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatRelativeTime(isoString: string | null | undefined): string {
  if (!isoString) return "Never";
  try {
    return formatDistanceToNow(parseISO(isoString), { addSuffix: true });
  } catch {
    return "Unknown";
  }
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function getChangeTypeColor(changeType: string): string {
  switch (changeType) {
    case "added": return "text-success";
    case "modified": return "text-warning";
    case "deleted": return "text-error";
    default: return "text-text-muted";
  }
}

export function getChangeTypeSymbol(changeType: string): string {
  switch (changeType) {
    case "added": return "+";
    case "modified": return "~";
    case "deleted": return "-";
    default: return "?";
  }
}
