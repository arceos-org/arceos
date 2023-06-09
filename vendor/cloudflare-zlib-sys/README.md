## [Cf-Zlib](https://github.com/cloudflare/zlib)

This crate has the same API as [zlib](https://zlib.net/), and conflicts with [libz-sys](https://crates.rs/crates/libz-sys).

It requires x86-64 CPU with SSE 4.2 or ARM64 with NEON & CRC. It does not support 32-bit CPUs at all.

## Cloning

This repository uses git submodules, so when cloning make sure to add `--recursive`

    git clone --recursive https://gitlab.com/kornelski/cloudflare-zlib-sys


## License

### Zlib

(C) 1995-2017 Jean-loup Gailly and Mark Adler

This software is provided 'as-is', without any express or implied
warranty.  In no event will the authors be held liable for any damages
arising from the use of this software.

Permission is granted to anyone to use this software for any purpose,
including commercial applications, and to alter it and redistribute it
freely, subject to the following restrictions:

1. The origin of this software must not be misrepresented; you must not
  claim that you wrote the original software. If you use this software
  in a product, an acknowledgment in the product documentation would be
  appreciated but is not required.
2. Altered source versions must be plainly marked as such, and must not be
  misrepresented as being the original software.
3. This notice may not be removed or altered from any source distribution.

Jean-loup Gailly jloup@gzip.org
Mark Adler madler@alumni.caltech.edu

If you use the zlib library in a product, we would appreciate *not* receiving
lengthy legal documents to sign.  The sources are provided for free but without
warranty of any kind.  The library has been entirely written by Jean-loup
Gailly and Mark Adler; it does not include third-party code.

If you redistribute modified sources, we would appreciate that you include in
the file ChangeLog history information documenting your changes.  Please read
the FAQ for more information on the distribution of modified source versions.

### libz-sys

This project is licensed under either of

  * [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
  * [MIT license](https://opensource.org/licenses/MIT)

at your option.
