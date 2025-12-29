## lighter-go signing libraries

Latest version: **v1.0.2**

Source: https://github.com/elliottech/lighter-go/releases/tag/v1.0.2

### Structure

- `linux/amd64/` - Linux x86_64 binaries
- `linux/arm64/` - Linux ARM64 binaries
- `darwin/arm64/` - macOS ARM64 binaries
- `windows/amd64/` - Windows x86_64 binaries

Each directory contains:
- `liblighter-signer.{so|dylib|dll}` - Platform-specific library
- `liblighter-signer.h` - C header file

### SHA256 Checksums

```
671bd5ba0b897395b999143bb6d666f043ff837c02cb0ab6cc4a10c84190816a  libs/linux/amd64/liblighter-signer.so
79fa94f3bb5a80fa6e56d578061fcba79eef21135f5c9aea4a92f2aca93adb86  libs/linux/amd64/liblighter-signer.h
7418a2dbeae0506c286a678d454d8b27b148f445b736f9886e89c61a2ffb3a13  libs/linux/arm64/liblighter-signer.so
79fa94f3bb5a80fa6e56d578061fcba79eef21135f5c9aea4a92f2aca93adb86  libs/linux/arm64/liblighter-signer.h
99c6bf2d8589ba2b5f687ada675c9cbd1764744a5ee323ba3b464c928772d170  libs/darwin/arm64/liblighter-signer.dylib
d6fff007db68316ab8e4d31c3356049980050ed2bb784f718bac43eeb615b9f9  libs/darwin/arm64/liblighter-signer.h
b7c3589d173f3adbb4b12b67595161ba954c04214def6beeb2e534b74ddeedbc  libs/windows/amd64/liblighter-signer.dll
d3b788b763a6bad811d2e10a9f5fba5ab83b4d3859b2ae53368b2e185d4c6e49  libs/windows/amd64/liblighter-signer.h
```

### Updating

Run `./update_libs.sh` to download the latest binaries from GitHub releases.

The script automatically:
- Downloads the latest release assets
- Verifies SHA256 checksums using digests from GitHub API
- Updates this README with the latest version and checksums
