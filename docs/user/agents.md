# Coding agents on the screen

The **Codex** and **Claude Code** widgets show what your coding agents are
doing while you work in another window: every project with its status, the
latest message, the tool in use and your account usage.

> **A first take.** The best way to show coding agents on a small screen is
> still being explored, so the look and the options of these widgets may change
> in the next releases. Ideas and feedback are welcome in the
> [issues](https://github.com/jripper77/bezel-evo/issues).

## Where the data comes from

Bezel reads two new local data sources:

- **Sessions.** The session logs that Codex and Claude Code write on your PC.
  For each project (a working folder, by its full path) Bezel follows the most
  recent session and reads its state: **working**, **waiting for you**,
  **finished**, **error** or **unknown**, with the last message, the current tool
  and how long ago it changed. Logs are read a little at a time, in a separate
  worker, and only while a widget on a screen or in Studio needs them.
- **Account usage.** The five-hour and weekly windows of your subscription,
  with the share used and the time left until each one resets. Codex records
  them in its own sessions. For Claude Code, Bezel asks the Claude client you
  are already signed in to (the command line or the VS Code extension) once a
  minute while a widget shows them. No prompt is sent to the model.

Nothing leaves your PC, and Bezel never reads, copies or stores credentials.
The log formats belong to the agents and may change with their updates; a value
Bezel cannot read is shown as unavailable, never guessed.

## Adding a widget

In **Widgets › Time and media**, drag **Codex** or **Claude Code** onto the
screen. Each widget follows one agent; add one of each to follow both.

Select the widget to see its general options:

- **Data**: an automatic list of all projects, or one project or session.
  How long finished projects stay listed, how long each page stays, which
  projects to show (click a project chip to hide it; its dot shows its state)
  and, for Claude Code, **Refresh Claude usage**.
- **Parts**: the title, the project rows and the two usage windows. The eye
  hides or shows a part; the arrow opens it.
- **Text**: the font, size and colour every part starts from.
- **Behavior**: **Highlight status changes** softly lights the row of a project
  whose state changed, for two seconds. Reduce motion turns it off.

## Editing one part

Double-click a part on the canvas, or open it from **Parts**. It is outlined in
orange and the inspector shows only that part:

- **Title**: its text (empty keeps the agent's name), the page number, and its
  own font, weight, colour and size.
- **Project rows**: what each row shows (status dot, status, time since the
  last update, the tool in use, row dividers, the full path), where the status
  goes (on the right or below the name), the row height, a style for the
  project name and one for the status, and the colour of each state.
- **Usage**: the value shown (remaining, used or both), the reset time, one or
  two lines, a continuous or segmented bar with its thickness and spacing, the
  text size, the bar, track and warning colours, and the remaining share below
  which the warning colour is used (0 turns it off). Both usage windows share
  these settings.

Press **Escape** or the back arrow to return to the whole widget; a second
Escape clears the selection. Click another part of the same widget to open it.

## Moving and resizing parts

With a part open, drag inside its outline to move it and drag its handles to
resize it. It stays inside the widget, and each drag is a single undo step.
Places are kept as shares of the widget, so the parts follow when you resize
the widget itself. The other parts keep their automatic place; when a usage bar
moves away, the project rows take the space it leaves.

**Layout › Place in the widget** shows the part's position and size in pixels,
and **Back to the automatic place** restores it. **Parts › Automatic layout for
every part** restores them all.

## Good to know

- Usage belongs to the account, so it is the same in every project of the
  same agent.
- Claude Code usage needs the Claude command line or the VS Code extension
  installed and signed in. If it is unavailable, check that you are signed in
  and use **Refresh Claude usage**.
- Last valid usage stays visible for up to 15 minutes if the client returns
  nothing, with its age after three minutes.
