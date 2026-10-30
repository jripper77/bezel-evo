# Running at sign-in

[Português (Brasil)](pt-BR/run-at-login.md)

**Bezel Light** keeps your saved themes on the screens without Studio. It lives
in the tray; its menu has **Open Studio** and **Quit**, and a double-click on the
icon opens Studio too.

## Light at Windows sign-in

Tick **Start Bezel Evo Light when I sign in** in Setup. When you sign in, Light starts with
the theme and the screens you last saved in Studio.

With the portable ZIP, double-click `start-light.cmd` to start Light; the
[Windows packaging guide](../../packaging/windows/README.md) has the command that
adds it to sign-in.

**Screen → Settings → Start with the computer** in Studio is the same option:
turning it on or off there does the same as the Setup checkbox. Studio itself
does not start at sign-in; an older Studio entry that did is switched to Light
the next time Studio opens.

## Studio and Light together

Light owns the screens: it runs the themes, reads the sensors, keeps the graph
history and the clocks, plays the transitions and decodes video backgrounds.
Studio is the editor that talks to it.

- **Opening Studio** does not interrupt anything: the screen keeps its picture,
  and Studio's canvas shows exactly what Light draws. Your edits go to Light as
  you make them, so a live screen follows them.
- **Closing Studio** keeps everything going: no redraw, graphs keep their
  history, a video keeps playing. If you close without saving, Studio asks
  first, and the screen goes back to the saved theme, still without starting
  over.
- The two talk over a local channel that only your Windows user can open.
- If Light cannot start or stops answering, Studio drives the screens itself,
  as earlier versions did.

Closing Studio always hands the screens over to Light; there is no option to
keep Studio in the tray instead. **Quit** in the tray menu stops Light; the
screen keeps its last picture.

## Multiple screens

The screen selector at the top of Studio opens a separate document for each
screen. Switching keeps each one's edits and Undo history, and the other screens
keep running. Save a theme and turn on **Live** on each screen you want to run.

Light runs all of them in one process, with one sensor reader and one worker per
screen, so a slow screen does not hold up a fast one.

## At shutdown

When Windows shuts down or restarts, Light stops drawing and applies each rev C
screen's [shutdown choice](power-off.md) (leave as it is, turn off, a stored
video or the photo album). Screens without those choices, such as the 3.5" rev A,
are turned off. Light waits at most 3.5 seconds; the screen's own sleep timer is
the fallback if one does not answer.

After a long pause (sleep, hibernation), Light restarts the sensor reader and
reconnects the screens by itself.

## Without Studio or Light

`bezel run` shows one theme from a terminal until Ctrl+C:

```powershell
bezel run turing-8.8-horizontal
```

It does not share the screen: quit Light first, or it refuses the port. A
scheduled task can start it at sign-in, but Light is the better way.
