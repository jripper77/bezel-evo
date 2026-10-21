# Profiles and automation

A **profile** says which theme each screen shows: a *Gaming* profile can put
FPS and GPU temperatures on the screen, a *Music* profile the media player.
Bezel can switch profiles by itself, from what you do on the PC, or keep the
one you pick.

Open **Themes → Profiles and automation** in Studio, or **Manage profiles** in
Light's tray menu.

## Who decides

On the left of the dialog:

- **Automatic**: rules choose the profile. When no rule holds, the screens
  show the profile chosen under **When no rule holds**, or the base themes.
- **Manual**: rules are paused and the screens always show the profile you
  pick. You can also change it from the tray.

**Base themes** are the themes chosen in Studio for each screen (the editor's
**Live**). They show when no profile is on.

Under them is the list of profiles, with a preview of what each shows and
**Active now** on the one the screens show at the moment. **New profile** adds
one; its **⋯** menu renames, duplicates or deletes it.

## A profile

A profile has three sections, which you can open and close:

- **Screens and themes**: the theme for each screen. A screen that is not
  connected keeps its theme for when it comes back.
- **Turns on when**: the rules, written as sentences you complete. Add one with
  the buttons under the list:
  - **App in foreground**: *when Steam is in front*;
  - **App running**: *when OBS is running*;
  - **Media playing**: *when music is playing*, in any player or in one app;
  - **Sensor**: *when GPU temperature is above 80 for 10 s*;
  - **Clock**: *on Monday to Friday, from 09:00 to 18:00*; a range can cross
    midnight;
  - **Idle**: *after 10 minutes without input*;
  - **PC locked**: *when the PC is locked*.

  Each rule has a **Priority** (0–100): when several profiles hold at once,
  the highest wins. **Combine these rules** decides whether **Any rule** or
  **All rules** must hold. Each rule says whether it is **True now**.
- **Behavior**: the **Return delay**, how long the profile stays on after its
  rules stop holding, and the **Transition between profiles**, the same for
  every profile: a curtain, a wipe, blinds or a mosaic closes over the screen
  while the new theme is prepared, then opens again (150 to 3000 ms; between
  1500 and 2500 ms the motion is smoothest). **Try the transition** plays it
  without saving. With **Base themes** selected, this section also holds
  **When no rule holds**.

**Activate for 30 s** shows the profile on the live screens for 30 seconds,
then they go back to what they showed. Nothing is saved.

Changes save by themselves (*Saved automatically*). A profile that is not
complete yet, such as a rule without an app, is not saved until you complete it;
closing the dialog then asks whether to discard the changes.

While you edit a theme in Studio, automatic changes pause on that screen.

The screen chip in Studio's status bar shows the active profile and whether it
was chosen by the rules or by hand.
