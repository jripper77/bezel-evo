// The statusbar's Screen chip (D-2026-10-20-studio-ux-v2-11, board
// StatusBarNew): what it says, its colour and the rows of its dialog, from
// the screen in use and the live state. Green while live, yellow when
// something needs a look, grey (no tone) while not live or without a screen.

/**
 * @param {{name?: string, port?: string, size?: {width: number, height: number}|null, turned?: boolean,
 *   live?: boolean, fps?: number|null, message?: string, problem?: boolean, theme?: string|null, profile?: string|null}|null} screen
 * @param {(key: string, params?: object) => string} t
 */
export function screenStatus(screen, t, { message = '', problem = false } = {}) {
  // Without a screen the chip is off, or yellow when finding screens failed.
  if (!screen) return { present: false, label: t('shell.noScreenChip'), tone: problem ? 'warn' : '', title: [t('shell.noScreenChip'), message].filter(Boolean).join('\n'), rows: [] };
  const live = Boolean(screen.live);
  const fps = live && Number.isFinite(screen.fps) ? Math.round(screen.fps) : null;
  const label = live
    ? fps === null ? screen.name : t('shell.screenChipLive', { name: screen.name, fps })
    : t('shell.screenChipIdle', { name: screen.name });
  const size = screen.size ? `${screen.size.width} × ${screen.size.height}` : null;
  const rows = [
    ['shell.screenLink', screen.port ? t('shell.screenUsb', { port: screen.port }) : null],
    ['shell.screenResolution', size && screen.turned ? t('shell.screenTurned', { size, degrees: 180 }) : size],
    ['shell.screenFrames', fps === null ? null : t('shell.fpsValue', { fps })],
    ['shell.screenTheme', live ? screen.theme ?? null : null],
    ['shell.screenProfile', screen.profile ?? null],
  ].filter(([, value]) => value).map(([key, value]) => ({ label: t(key), value }));
  return {
    present: true,
    label,
    tone: problem || screen.problem ? 'warn' : live ? 'ok' : '',
    title: [label, message || screen.message].filter(Boolean).join('\n'),
    rows,
    live,
  };
}

/**
 * The profile the engine has active now (bezel-runtime ProfileStatus, T-6),
 * by name and whether the rules or the user chose it; null when the base
 * themes show or the engine says nothing: the UI never invents one.
 */
export function activeProfile(overview, t) {
  const status = overview?.status;
  const id = status?.active?.profile;
  const profile = id ? overview.config?.profiles?.find(p => p.id === id) : null;
  if (!profile) return null;
  const name = profile.name || t('profiles.new');
  const mode = t(status.automatic ? 'shell.profileAuto' : 'shell.profileManual');
  return { name, mode, text: t('shell.profileActive', { name, mode }) };
}
