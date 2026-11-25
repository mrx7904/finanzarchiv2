# #08 Releases built on the owner's machines

**Goal:** One release script builds the Windows installer on the PC and another the dmg on the Mac, each after the check, and both upload to one GitHub release.
**Requirements**: FA-12 · **Depends on**: #01, #06

## Done when

- [x] The version lives in one place and the scripts read it from there.
- [ ] The Windows script builds the installer, uploads it with `gh`, and the app installs and starts on the PC.
- [ ] The macOS script builds the dmg for Apple silicon, signed ad hoc, uploads it to the same release, and the app opens on the Mac after the download.
- [ ] The size of both installers stands in `decision-stack.md`.
- [x] No file under `.github/workflows/` exists.

## Out of scope

- Updates from inside the app, belong to stage 6
- Signing with an Apple Developer ID, the owner decides
