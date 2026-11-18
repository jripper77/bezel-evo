import { sections as agentSections } from './agent.js';

/** Provider-specific details, sharing the project and display controls. */
export function sections(ctx) {
  return agentSections(ctx, 'codex');
}
