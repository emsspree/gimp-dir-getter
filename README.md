# A `gimp-directory` Getter
This helper, written in Rust, looks for GIMP 3.x
  [settings/configuration directories](https://docs.gimp.org/3.0/en/gimp-fire-up.html#gimp-concepts-setup)
  (a.k.a. [`gimp-directory`](https://developer.gimp.org/api/3.0/libgimp/func.directory.html))
  and prints their paths.
It supports filtering
  by release cycles (even/odd),
  by version number, and 
  by tags for installation sources.

| Options:         |   |
| --------         | - |
| `-v`, `--version`| Show program version
| `-h`, `--help`   | Show help message
| `--even`         | Only show even minor versions (e.g., 3.0, 3.2, 3.4).<br>Stable GIMP release versions always have [even minor versions](https://developer.gimp.org/core/maintainer/versioning/). |
| `--odd`          | Only show odd minor versions (e.g., 3.1, 3.3, 3.5). |
| `--only`         | Only include specific version numbers and/or tags.<br>Examples:<br>`gimp-dir-getter --only=flatpak`<br>`gimp-dir-getter --only=3.0 --only=3.4,snap` |
| `--ignore`       | Exclude specific versions or tags. <br>Examples:<br>`gimp-dir-getter --ignore=flatpak`<br>`gimp-dir-getter --ignore=3.0 --ignore=3.4,snap` |
| **Tags:**        | xdg, flatpak, snap, macos, windows |
