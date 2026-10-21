# Credits and third-party licences

Bezel Evo is a fork of [Bezel by Alison Amorim (@slipalison)](https://github.com/slipalison/bezel).
Original code, editor, protocol research, themes and inherited documentation
retain their authorship and copyright notices. The Bezel Evo name identifies
this fork; it does not replace those credits.

Bezel and this fork are licensed under [GPL-3.0-or-later](LICENSE), as declared
in [Cargo.toml](Cargo.toml). Component licences remain in force.

| Component | Licence | Notices and source |
| --- | --- | --- |
| LibreHardwareMonitorLib | MPL-2.0 | [Helper documentation](apps/bezel-sensors-helper/README.md), including original library source, dependency licences and source metadata |
| Tabler Icons | MIT | [Licence](apps/bezel-studio/src/assets/tabler/LICENSE) |
| Material Design Icons (hardware subset) | Apache-2.0 | [Licence](apps/bezel-studio/src/assets/mdi/LICENSE) and [NOTICE](apps/bezel-studio/src/assets/mdi/NOTICE) |
| IBM Plex Sans / Mono (Studio's interface font) | SIL OFL 1.1 | [Licence](apps/bezel-studio/src/assets/fonts/ibm-plex/LICENSE), [sources and SHA-256 checksums](apps/bezel-studio/src/assets/fonts/ibm-plex/README.md) |
| Inter, JetBrains Mono, Doto, Doto Rounded, IBM Plex Mono, Fraunces, Instrument Sans, Space Grotesk, Bricolage Grotesque (theme fonts, embedded unmodified in `bezel.exe` and `bezel-studio.exe`, and shipped in `themes/fonts`) | SIL OFL 1.1 | [Inter licence](themes/fonts/Inter-OFL.txt), [JetBrains Mono licence](themes/fonts/JetBrainsMono-OFL.txt), [Doto licence](themes/fonts/Doto-OFL.txt), [IBM Plex Mono licence](themes/fonts/IBMPlexMono-OFL.txt), [Fraunces licence](themes/fonts/Fraunces-OFL.txt), [Instrument Sans licence](themes/fonts/InstrumentSans-OFL.txt), [Space Grotesk licence](themes/fonts/SpaceGrotesk-OFL.txt), [Bricolage Grotesque licence](themes/fonts/BricolageGrotesque-OFL.txt) |
| Tauri Windows installer template | MIT | [Packaging notes](packaging/windows/README.md); the licence is included in Windows packages |

The OFL lets these fonts be bundled and embedded with software; they are not
sold on their own, and their licence files ship with the Windows packages. Fonts
you add to your own fonts folder or include in a theme keep their own licences:
Bezel cannot grant redistribution rights for them (see [Fonts](docs/user/fonts.md)).

The bundled sensor helper includes Libre's original MPL licence and notices,
including notices for embedded PawnIO modules. Dependency versions, hashes,
NuGet metadata and original source references are documented in the
[helper guide](apps/bezel-sensors-helper/README.md).

Weather data comes from [Open-Meteo](https://open-meteo.com/) under
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/); geocoding data comes
from [GeoNames](https://www.geonames.org/). Current conditions are model-based.
See [Open-Meteo's usage terms](https://open-meteo.com/en/terms) and the
[fork notes](FORK.md) for attribution and public/commercial use.

Protocol knowledge also draws on
[turing-smart-screen-python](https://github.com/mathoudebine/turing-smart-screen-python)
and the inherited [protocol research](docs/reverse-engineering/README.md).
