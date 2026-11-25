# #08 Releases built on the owner's machines

**Goal:** One release script builds the Windows installer on the PC and another the dmg on the Mac, each after the check, and both upload to one GitHub release.
**Requirements**: FA-12 · **Depends on**: #01, #06

## Done when

- [x] The version lives in one place and the scripts read it from there.
- [x] The Windows script builds the installer, uploads it with `gh`, and the app installs and starts on the PC. Draft v0.1.0 with `Finanzarchiv.2_0.1.0_x64-setup.exe` (6,470,286 bytes) uploaded on 2026-11-25 from commit 6d22bcf.
- [ ] The macOS script builds the dmg for Apple silicon, signed ad hoc, uploads it to the same release, and the app opens on the Mac after the download.
- [ ] The size of both installers stands in `decision-stack.md`.
- [x] No file under `.github/workflows/` exists.

## Out of scope

- Updates from inside the app, belong to stage 6
- Signing with an Apple Developer ID, the owner decides
