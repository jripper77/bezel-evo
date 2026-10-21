// Portable image controls read real asset metadata; no video playback until T-11.
export const IMAGE_ANCHORS = Object.freeze(['topLeft', 'top', 'topRight', 'left', 'center', 'right', 'bottomLeft', 'bottom', 'bottomRight']);
export const isAnimatedImage = (ref, assets = []) => /\.gif$/i.test(ref) || assets.some(asset => asset.ref === ref && asset.kind === 'image' && asset.animated === true);
export const playbackSpeeds = speed => [...new Set([0.5, 1, 2, ...(Number.isFinite(speed) && speed > 0 ? [speed] : [])])].sort((a, b) => a - b);
