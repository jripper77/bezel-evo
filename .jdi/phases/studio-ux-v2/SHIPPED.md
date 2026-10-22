shipped_at: 2026-10-22T21:49:40Z
verdict: APPROVED
by: jripper77

## Learnings
- Run the full e2e suite on the integrated base before asking for manual approval: 33 failures came only from specs left on superseded controls.
- When a status chip and its popover share data, update the source before projecting the popover; test the first opening, not a later refresh.
- Drag tests across tall panels must scroll the drop target into view during the drag before measuring it.
