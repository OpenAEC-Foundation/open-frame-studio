// Gross envelope used by the dashboard, including the bounding corners of curved frames.
export function frameEnvelopeAreaM2(frame) {
  const width = frame?.outerWidth;
  const height = frame?.outerHeight;
  if (!Number.isFinite(width) || !Number.isFinite(height) || width <= 0 || height <= 0) return null;
  const area = width * height / 1e6;
  return Number.isFinite(area) ? area : null;
}
