export const TRACKED_SOURCES: readonly string[];
export function computeStamp(root?: string): { stamp: string; sources: Record<string, string> };
