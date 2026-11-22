# Packaging & Universal Linux Distribution

Find-Me follows modern FreeDesktop XDG specifications and supports all major Linux packaging formats.

---

## 1. Supported Packaging Formats

| Format | Target Distributions | Build Script | Output Path |
| :--- | :--- | :--- | :--- |
| **Debian Package (`.deb`)** | Debian, Ubuntu, Linux Mint, Pop!_OS | `scripts/package_deb.sh` | `target/debian/find-me_*.deb` |
| **Portable AppImage** | Any Linux (GLIBC 2.31+) | `scripts/package_appimage.sh` | `target/appimage/Find-Me-x86_64.AppImage` |
| **Flatpak Bundle** | Flathub, Fedora Silverblue, SteamOS | `packaging/flatpak/` | Flatpak Sandbox Bundle |
| **Arch Linux AUR** | Arch Linux, Manjaro, EndeavourOS | `packaging/aur/` | `PKGBUILD`, `.SRCINFO` |

---

## 2. Building Packages

### Debian Package (`.deb`)
Requires `cargo-deb`:
```bash
cargo install cargo-deb
bash scripts/package_deb.sh
```
* Generates `.deb` package containing the stripped release binary, manual page, desktop launcher, icons, and AppStream metadata.
* Enforces POSIX file permissions (`0755` executable, `0644` data).

### Standalone AppImage
```bash
bash scripts/package_appimage.sh
```
* Bundles all necessary dynamic libraries using `appimagetool`.
* Runs on any Linux distribution without system installation.

### Flatpak
* Manifest: `packaging/flatpak/io.github.faizan_nadeem.find_me.yaml`
* Sandbox Permissions:
  - `--filesystem=home:ro` (Read-only access to user directories for indexing).
  - `--talk-name=org.freedesktop.FileManager1` (DBus file manager highlighting).
  - `--socket=wayland`, `--socket=x11` (Display server connections).

### Arch Linux AUR (`PKGBUILD`)
* Directory: `packaging/aur/PKGBUILD`
* Validated with `makepkg` and `namcap`.

---

## 3. FreeDesktop XDG Compliance

Find-Me installs standards-compliant desktop integration files:

| File Type | Installation Location | Purpose |
| :--- | :--- | :--- |
| **Desktop Launcher** | `~/.local/share/applications/find-me.desktop` | Application menu visibility and launch command. |
| **Scalable Vector Icon** | `~/.local/share/icons/hicolor/scalable/apps/find-me.svg` | Crisp UI icon at any display scale (100%–300% HiDPI). |
| **AppStream Metadata** | `~/.local/share/metainfo/io.github.faizan_nadeem.find_me.metainfo.xml` | Software Center / GNOME Software / Discover catalog listing. |
| **MIME Associations** | `~/.local/share/mime/packages/find-me.xml` | Custom Everything file list (`.efu`) file associations. |

---

## 4. Automated User Installation (`install_user.sh`)

To install or update Find-Me in your personal user environment:
```bash
bash scripts/install_user.sh
```
This requires **zero root privileges** and configures desktop launchers and icon databases automatically.
