import { extensionOf, fileNameOf, formatDuration } from './editor/background.js';
// Portable image controls read real asset metadata; no video playback until T-11.
export const IMAGE_ANCHORS = Object.freeze(['topLeft', 'top', 'topRight', 'left', 'center', 'right', 'bottomLeft', 'bottom', 'bottomRight']);
export const isAnimatedImage = (ref, assets = []) => /\.gif$/i.test(ref) || assets.some(asset => asset.ref === ref && asset.kind === 'image' && asset.animated === true);
export const playbackSpeeds = speed => [...new Set([0.5, 1, 2, ...(Number.isFinite(speed) && speed > 0 ? [speed] : [])])].sort((a, b) => a - b);

/** Facts of a real theme asset; an absent file never gets invented metadata. */
export function imageAssetFacts(ref, assets = []) {
  const asset = assets.find(item => item.ref === ref);
  return { name: fileNameOf(ref), format: extensionOf(ref).toUpperCase(),
    preview: asset?.poster ? assets.find(item => item.ref === asset.poster)?.dataUrl ?? asset.dataUrl ?? null : asset?.dataUrl ?? null,
    bytes: Number.isFinite(asset?.bytes) ? asset.bytes : null,
    duration: formatDuration(asset?.durationMs),
  };
}
