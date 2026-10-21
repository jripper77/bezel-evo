# Fonts

Studio and Light draw text with the same fonts, from three places. The
[font picker](studio-editing.md#fonts) lists each of them in its own section.

## Letter case

Select a text widget and open **Text → Letter case**. Choose Original,
UPPERCASE, lowercase, First letter uppercase or Capitalize Each Word. The same
control applies to values, clocks, weather and media metadata.

Formatting changes the displayed text, including accented letters, without
rewriting its source. Original restores the source's case. Changes are undoable
and saved with the theme. **Fit content** measures the formatted text, so it also
accounts for letters becoming wider after conversion.

## Included with Bezel

**Inter**, **JetBrains Mono**, **Doto**, **Doto Rounded** and **IBM Plex Mono**
are built into Bezel Evo: every theme can use them on any PC, with nothing to
install. Doto is a dot-matrix face that suits clocks and big readings; IBM Plex
Mono is the typeface of Studio's own interface. The built-in themes use them.

## My fonts

Use a font without installing it in Windows: copy its `.ttf`, `.otf` or `.ttc`
file into your fonts folder,

```text
%APPDATA%\Bezel Evo\fonts
```

**Open fonts folder** in the font picker opens it (and creates it the first
time). Studio and Light notice new or changed files within a couple of seconds;
**Reload fonts** refreshes a list that is already open. The folder belongs to
your Windows user and stays when you update or reinstall Bezel. Subfolders are
not read.

Fonts installed in Windows appear under **System fonts** as usual.

## Fonts packed inside a theme

A theme you share should carry its fonts, so it looks the same on another PC.
Select the text, open the font picker, highlight the font and choose **Include
in theme**: Bezel copies the font file into the theme. Fonts held by the theme
appear under **Theme fonts** and win over a font with the same name on the PC.

Read the font's licence first: some fonts do not allow redistribution, and Bezel
cannot grant that right. The fonts included with Bezel are not copied into
themes, since every Bezel has them.

**Remove font from theme** takes the file out again in one undoable step; the
text keeps the family name, weight and slant, and uses the font of that name if
the PC has it. Saving drops font files no text uses any more.

## A font this PC does not have

A theme can name a font that is neither included, in your folder, nor installed.
The font picker then marks it **Not installed** and names the font the screen
shows instead. Install the font, copy it into your fonts folder, or pick another.

Limits and technical details (sizes, number of files, how files are checked) are
in [Fonts in Bezel Evo](../font-levels.md).
