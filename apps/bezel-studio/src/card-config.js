// Canonical Card data shared by editor commands, playback and controls.
/**
 * A Card as the editor keeps it: legacy `rotationSeconds` becomes a timed
 * trigger, and per-pair `pairTransitions` (development builds only, D-24) is
 * dropped, so the Card's own `transition` serves every face change.
 */
export function normalizeCard(card) {
  const legacy = card.rotationSeconds != null;
  if (!legacy && !('pairTransitions' in card)) return card;
  const { pairTransitions: _ignored, ...kept } = card;
  if (!legacy) return kept;
  const { rotationSeconds, ...rest } = kept;
  return { ...rest, triggers: [...(card.triggers ?? []), {
    source: 'timer', intervalSeconds: rotationSeconds, nextFace: true,
    app: '', face: 0, priority: 0, returnSeconds: 0, returnFace: null,
  }] };
}

export function normalizeCardTheme(theme) {
  const elements = theme.elements.map(e => {
    if (!e.card) return e;
    const card = normalizeCard(e.card);
    return card === e.card ? e : { ...e, card };
  });
  return elements.every((e, i) => e === theme.elements[i]) ? theme : { ...theme, elements };
}

/** A Card transition's duration bounds, ms (slider and number alike). */
export const CARD_DURATION = Object.freeze({ min: 150, max: 1500, step: 50 });

/** A typed duration on the Card's scale: within bounds, on a 50 ms step. */
export function cardDuration(ms) {
  const { min, max, step } = CARD_DURATION;
  const value = Number.isFinite(ms) ? ms : min;
  return Math.min(max, Math.max(min, Math.round(value / step) * step));
}

/** The transition with another effect, keeping its direction, duration and shared-background choice (defaults when new). */
export function withCardEffect(settings, effect) {
  return { effect, direction: settings?.direction ?? 'left', durationMs: settings?.durationMs ?? 650, includeBase: settings?.includeBase ?? true };
}

/** An isolated face preview; the edited document and its playback state stay untouched. */
export function cardFacePreviewTheme(theme, id, face) {
  const element = theme.elements.find(element => element.id === id);
  if (!element?.card || !Number.isInteger(face) || face < 0 || face >= element.card.faces.length) return null;
  return { ...theme, elements: theme.elements.map(item => item === element ? { ...item, card: { ...item.card, activeFace: face } } : item) };
}
