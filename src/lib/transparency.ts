export const ALPHA_MIN = 25;
export const ALPHA_MAX = 255;

export function alphaToPercentLabel(alpha: number): string {
  const clamped = Math.min(Math.max(alpha, ALPHA_MIN), ALPHA_MAX);
  return ((clamped / 255) * 100).toFixed(0);
}

export function percentToAlpha(percent: number): number {
  const clampedPercent = Math.min(Math.max(percent, 0), 100);
  return Math.round((clampedPercent / 100) * 255);
}
