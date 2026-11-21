# Third-Party Notices

**FrameLab — licences and notices for third-party components**

Last updated: 21 November 2026

FrameLab is distributed under the [MIT License](LICENSE). It also includes, links against and is built
with software written by other people. Those components remain under their own licences, and those
licences — not FrameLab's — govern the components.

Nothing in this file changes the terms of any third-party licence. Where a component is offered under
a choice of licences, FrameLab uses it under the MIT option unless stated otherwise.

> **Not legal advice.** This inventory was compiled from the actual package metadata and licence files
> of the versions FrameLab builds with. Have it reviewed before a commercial release, and regenerate it
> whenever dependencies change.

---

## 1. Components distributed with the application

These components are copied onto the user's computer by the installer and are therefore redistributed.

### Intel PresentMon 2.6.0 — `presentmon.exe`

- **Project:** PresentMon — <https://github.com/GameTechDev/PresentMon>
- **Copyright:** Copyright (C) 2017-2024 Intel Corporation
- **Licence:** MIT
- **How it is used:** an unmodified release binary, launched as a separate process to read frame
  presentation events. It is stored in the application's resources directory together with
  `LICENSE.txt` and a pinned `SHA256.txt`, and FrameLab refuses to run it if the hash does not match.
- **Obligation:** the copyright notice and permission notice are reproduced in full in
  [section 6.1](#61-intel-presentmon-mit) and are also shipped next to the binary as
  `binaries/presentmon/LICENSE.txt`.

PresentMon is a trademark of Intel Corporation. FrameLab is not affiliated with, endorsed by or
sponsored by Intel.

### Inter (variable) typeface — `@fontsource-variable/inter` 5.3.0

- **Project:** Inter — <https://github.com/rsms/inter>
- **Copyright:** Copyright 2016 The Inter Project Authors
- **Licence:** SIL Open Font License, Version 1.1
- **How it is used:** the Latin, Greek, Cyrillic and Vietnamese variable font files are bundled with
  the interface build so that FrameLab renders identically offline. The fonts are unmodified and are
  not sold separately.
- **Obligation:** the copyright notice and the full licence are reproduced in
  [section 6.4](#64-sil-open-font-license-11). The font is not renamed, and no reserved font name is
  used for a modified version, because no modified version is distributed.

### FrameLab artwork

The application icon set, the wordmark and the in-app mark were created for this project. They are
covered by FrameLab's own MIT licence and contain no third-party assets.

---

## 2. Libraries linked into the desktop application

These Rust crates are compiled into the executable.

| Component | Version | Licence |
| --- | --- | --- |
| Tauri | 2.12.7 | MIT OR Apache-2.0 |
| tauri-build (build time) | 2.7.6 | MIT OR Apache-2.0 |
| tauri-plugin-autostart | 2.7.0 | MIT OR Apache-2.0 |
| tauri-plugin-dialog | 2.8.4 | MIT OR Apache-2.0 |
| tauri-plugin-opener | 2.7.3 | MIT OR Apache-2.0 |
| tauri-plugin-single-instance | 2.5.5 | MIT OR Apache-2.0 |
| tauri-plugin-window-state | 2.5.0 | MIT OR Apache-2.0 |
| serde, serde_json | 1.0.x | MIT OR Apache-2.0 |
| sha2 | 0.10.9 | MIT OR Apache-2.0 |
| thiserror | 1.0.69 | MIT OR Apache-2.0 |
| chrono | 0.4.45 | MIT OR Apache-2.0 |
| windows-sys | 0.45.0 | MIT OR Apache-2.0 |
| rusqlite (bundled SQLite) | 0.32.1 | MIT |
| sysinfo | 0.33.1 | MIT |
| tokio | 1.54.1 | MIT |

SQLite, embedded through `rusqlite`'s bundled build, is in the public domain
(<https://www.sqlite.org/copyright.html>).

---

## 3. Libraries bundled into the interface build

These packages are compiled into the JavaScript and CSS that ships inside the application.

| Component | Version | Licence |
| --- | --- | --- |
| react, react-dom | 19.3.1 | MIT |
| react-router-dom | 7.18.6 | MIT |
| recharts | 3.11.0 | MIT |
| zustand | 5.1.0 | MIT |
| motion | 14.14.0 | MIT |
| clsx | 2.1.1 | MIT |
| tailwind-merge | 3.7.1 | MIT |
| lucide-react | 1.77.0 | ISC |
| @tauri-apps/api | 2.12.5 | Apache-2.0 OR MIT |
| @radix-ui/react-dialog, -dropdown-menu, -label, -popover, -progress, -scroll-area, -select, -separator, -slider, -slot, -switch, -tabs, -tooltip | 1.3.x – 2.4.x | MIT |
| class-variance-authority | 0.7.1 | Apache-2.0 |
| @fontsource-variable/inter | 5.3.0 | OFL-1.1 |

### Build-only tools (not distributed)

FrameLab is built with the following tools. They are not included in the installed application:

| Component | Licence |
| --- | --- |
| Vite | MIT |
| TypeScript | Apache-2.0 |
| Tailwind CSS and @tailwindcss/vite | MIT |
| @vitejs/plugin-react | MIT |
| sharp (icon generation) | Apache-2.0 |
| @types/node, @types/react, @types/react-dom | MIT |
| NSIS (installer generator) | zlib/libpng |

NSIS is © 1999-2026 NSIS Contributors and is used to produce the Windows installer. The NSIS licence
does not require attribution in the installed product; it is acknowledged here voluntarily.

---

## 4. Microsoft components

### Microsoft Edge WebView2 Runtime

FrameLab's user interface is rendered with the Microsoft Edge WebView2 Runtime.

- **Supplier:** Microsoft Corporation
- **Licence:** Microsoft Software Licence Terms (proprietary, redistributable), not an open-source
  licence
- **Distribution:** FrameLab does not bundle the runtime. It relies on the Evergreen Runtime already
  present on current Windows 10 and Windows 11 systems, and the installer can download the Microsoft
  bootstrapper when it is missing.
- **Terms:** <https://learn.microsoft.com/microsoft-edge/webview2/>

The WebView2 Runtime may communicate with Microsoft services under Microsoft's own terms. See
[PRIVACY.md](PRIVACY.md) section 3.

### Windows platform components

FrameLab calls documented Windows APIs (performance counters, registry, display configuration, shell
launch, event tracing). Use of those APIs is governed by the Microsoft licence for the operating
system on which FrameLab runs.

---

## 5. Notes on obligations

- **MIT-licensed components (including PresentMon):** the copyright notice and permission notice must
  accompany copies. They are reproduced in section 6.1 and 6.2, and Intel's notice also ships beside
  the binary.
- **Apache-2.0 components:** a copy of the Apache License 2.0 is included in section 6.3. FrameLab does
  not modify those components. No NOTICE file was supplied by the versions in use that requires
  additional attribution beyond the copyright notices in this document.
- **SIL OFL 1.1 (Inter):** the font is distributed unmodified and bundled with software, which the
  licence expressly permits; the full licence and copyright notice are in section 6.4.
- **ISC (lucide-react):** permission notice retained in section 6.5.
- **No endorsement:** the presence of a component does not imply that its authors endorse FrameLab.

If you believe a component is missing from this list, contact **nuxxzul@gmail.com** so it can be
corrected in the next release.

---

## 6. Full licence texts

### 6.1 Intel PresentMon (MIT)

```text
MIT License

Copyright (C) 2017-2024 Intel Corporation

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### 6.2 MIT-licensed libraries

The MIT text in section 6.1 applies to each MIT-licensed component listed in sections 2 and 3, with
the copyright held by that component's respective authors as stated in its own source distribution.
FrameLab's own MIT licence, with its own copyright line, is in [LICENSE](LICENSE).

### 6.3 Apache License 2.0

```text
Apache License
                           Version 2.0, January 2004
                        http://www.apache.org/licenses/

   TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

   1. Definitions.

      "License" shall mean the terms and conditions for use, reproduction,
      and distribution as defined by Sections 1 through 9 of this document.

      "Licensor" shall mean the copyright owner or entity authorized by
      the copyright owner that is granting the License.

      "Legal Entity" shall mean the union of the acting entity and all
      other entities that control, are controlled by, or are under common
      control with that entity. For the purposes of this definition,
      "control" means (i) the power, direct or indirect, to cause the
      direction or management of such entity, whether by contract or
      otherwise, or (ii) ownership of fifty percent (50%) or more of the
      outstanding shares, or (iii) beneficial ownership of such entity.

      "You" (or "Your") shall mean an individual or Legal Entity
      exercising permissions granted by this License.

      "Source" form shall mean the preferred form for making modifications,
      including but not limited to software source code, documentation
      source, and configuration files.

      "Object" form shall mean any form resulting from mechanical
      transformation or translation of a Source form, including but
      not limited to compiled object code, generated documentation,
      and conversions to other media types.

      "Work" shall mean the work of authorship, whether in Source or
      Object form, made available under the License, as indicated by a
      copyright notice that is included in or attached to the work
      (an example is provided in the Appendix below).

      "Derivative Works" shall mean any work, whether in Source or Object
      form, that is based on (or derived from) the Work and for which the
      editorial revisions, annotations, elaborations, or other modifications
      represent, as a whole, an original work of authorship. For the purposes
      of this License, Derivative Works shall not include works that remain
      separable from, or merely link (or bind by name) to the interfaces of,
      the Work and Derivative Works thereof.

      "Contribution" shall mean any work of authorship, including
      the original version of the Work and any modifications or additions
      to that Work or Derivative Works thereof, that is intentionally
      submitted to Licensor for inclusion in the Work by the copyright owner
      or by an individual or Legal Entity authorized to submit on behalf of
      the copyright owner. For the purposes of this definition, "submitted"
      means any form of electronic, verbal, or written communication sent
      to the Licensor or its representatives, including but not limited to
      communication on electronic mailing lists, source code control systems,
      and issue tracking systems that are managed by, or on behalf of, the
      Licensor for the purpose of discussing and improving the Work, but
      excluding communication that is conspicuously marked or otherwise
      designated in writing by the copyright owner as "Not a Contribution."

      "Contributor" shall mean Licensor and any individual or Legal Entity
      on behalf of whom a Contribution has been received by Licensor and
      subsequently incorporated within the Work.

   2. Grant of Copyright License. Subject to the terms and conditions of
      this License, each Contributor hereby grants to You a perpetual,
      worldwide, non-exclusive, no-charge, royalty-free, irrevocable
      copyright license to reproduce, prepare Derivative Works of,
      publicly display, publicly perform, sublicense, and distribute the
      Work and such Derivative Works in Source or Object form.

   3. Grant of Patent License. Subject to the terms and conditions of
      this License, each Contributor hereby grants to You a perpetual,
      worldwide, non-exclusive, no-charge, royalty-free, irrevocable
      (except as stated in this section) patent license to make, have made,
      use, offer to sell, sell, import, and otherwise transfer the Work,
      where such license applies only to those patent claims licensable
      by such Contributor that are necessarily infringed by their
      Contribution(s) alone or by combination of their Contribution(s)
      with the Work to which such Contribution(s) was submitted. If You
      institute patent litigation against any entity (including a
      cross-claim or counterclaim in a lawsuit) alleging that the Work
      or a Contribution incorporated within the Work constitutes direct
      or contributory patent infringement, then any patent licenses
      granted to You under this License for that Work shall terminate
      as of the date such litigation is filed.

   4. Redistribution. You may reproduce and distribute copies of the
      Work or Derivative Works thereof in any medium, with or without
      modifications, and in Source or Object form, provided that You
      meet the following conditions:

      (a) You must give any other recipients of the Work or
          Derivative Works a copy of this License; and

      (b) You must cause any modified files to carry prominent notices
          stating that You changed the files; and

      (c) You must retain, in the Source form of any Derivative Works
          that You distribute, all copyright, patent, trademark, and
          attribution notices from the Source form of the Work,
          excluding those notices that do not pertain to any part of
          the Derivative Works; and

      (d) If the Work includes a "NOTICE" text file as part of its
          distribution, then any Derivative Works that You distribute must
          include a readable copy of the attribution notices contained
          within such NOTICE file, excluding those notices that do not
          pertain to any part of the Derivative Works, in at least one
          of the following places: within a NOTICE text file distributed
          as part of the Derivative Works; within the Source form or
          documentation, if provided along with the Derivative Works; or,
          within a display generated by the Derivative Works, if and
          wherever such third-party notices normally appear. The contents
          of the NOTICE file are for informational purposes only and
          do not modify the License. You may add Your own attribution
          notices within Derivative Works that You distribute, alongside
          or as an addendum to the NOTICE text from the Work, provided
          that such additional attribution notices cannot be construed
          as modifying the License.

      You may add Your own copyright statement to Your modifications and
      may provide additional or different license terms and conditions
      for use, reproduction, or distribution of Your modifications, or
      for any such Derivative Works as a whole, provided Your use,
      reproduction, and distribution of the Work otherwise complies with
      the conditions stated in this License.

   5. Submission of Contributions. Unless You explicitly state otherwise,
      any Contribution intentionally submitted for inclusion in the Work
      by You to the Licensor shall be under the terms and conditions of
      this License, without any additional terms or conditions.
      Notwithstanding the above, nothing herein shall supersede or modify
      the terms of any separate license agreement you may have executed
      with Licensor regarding such Contributions.

   6. Trademarks. This License does not grant permission to use the trade
      names, trademarks, service marks, or product names of the Licensor,
      except as required for reasonable and customary use in describing the
      origin of the Work and reproducing the content of the NOTICE file.

   7. Disclaimer of Warranty. Unless required by applicable law or
      agreed to in writing, Licensor provides the Work (and each
      Contributor provides its Contributions) on an "AS IS" BASIS,
      WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
      implied, including, without limitation, any warranties or conditions
      of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
      PARTICULAR PURPOSE. You are solely responsible for determining the
      appropriateness of using or redistributing the Work and assume any
      risks associated with Your exercise of permissions under this License.

   8. Limitation of Liability. In no event and under no legal theory,
      whether in tort (including negligence), contract, or otherwise,
      unless required by applicable law (such as deliberate and grossly
      negligent acts) or agreed to in writing, shall any Contributor be
      liable to You for damages, including any direct, indirect, special,
      incidental, or consequential damages of any character arising as a
      result of this License or out of the use or inability to use the
      Work (including but not limited to damages for loss of goodwill,
      work stoppage, computer failure or malfunction, or any and all
      other commercial damages or losses), even if such Contributor
      has been advised of the possibility of such damages.

   9. Accepting Warranty or Additional Liability. While redistributing
      the Work or Derivative Works thereof, You may choose to offer,
      and charge a fee for, acceptance of support, warranty, indemnity,
      or other liability obligations and/or rights consistent with this
      License. However, in accepting such obligations, You may act only
      on Your own behalf and on Your sole responsibility, not on behalf
      of any other Contributor, and only if You agree to indemnify,
      defend, and hold each Contributor harmless for any liability
      incurred by, or claims asserted against, such Contributor by reason
      of your accepting any such warranty or additional liability.

   END OF TERMS AND CONDITIONS

   Copyright 2022 Joe Bell

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
```

### 6.4 SIL Open Font License 1.1

```text
Copyright 2016 The Inter Project Authors (https://github.com/rsms/inter) Inter-Italic[opsz,wght].ttf: Copyright 2016 The Inter Project Authors (https://github.com/rsms/inter)

This Font Software is licensed under the SIL Open Font License, Version 1.1.
This license is copied below, and is also available with a FAQ at:
http://scripts.sil.org/OFL


-----------------------------------------------------------
SIL OPEN FONT LICENSE Version 1.1 - 26 February 2007
-----------------------------------------------------------

PREAMBLE
The goals of the Open Font License (OFL) are to stimulate worldwide
development of collaborative font projects, to support the font creation
efforts of academic and linguistic communities, and to provide a free and
open framework in which fonts may be shared and improved in partnership
with others.

The OFL allows the licensed fonts to be used, studied, modified and
redistributed freely as long as they are not sold by themselves. The
fonts, including any derivative works, can be bundled, embedded,
redistributed and/or sold with any software provided that any reserved
names are not used by derivative works. The fonts and derivatives,
however, cannot be released under any other type of license. The
requirement for fonts to remain under this license does not apply
to any document created using the fonts or their derivatives.

DEFINITIONS
"Font Software" refers to the set of files released by the Copyright
Holder(s) under this license and clearly marked as such. This may
include source files, build scripts and documentation.

"Reserved Font Name" refers to any names specified as such after the
copyright statement(s).

"Original Version" refers to the collection of Font Software components as
distributed by the Copyright Holder(s).

"Modified Version" refers to any derivative made by adding to, deleting,
or substituting -- in part or in whole -- any of the components of the
Original Version, by changing formats or by porting the Font Software to a
new environment.

"Author" refers to any designer, engineer, programmer, technical
writer or other person who contributed to the Font Software.

PERMISSION & CONDITIONS
Permission is hereby granted, free of charge, to any person obtaining
a copy of the Font Software, to use, study, copy, merge, embed, modify,
redistribute, and sell modified and unmodified copies of the Font
Software, subject to the following conditions:

1) Neither the Font Software nor any of its individual components,
in Original or Modified Versions, may be sold by itself.

2) Original or Modified Versions of the Font Software may be bundled,
redistributed and/or sold with any software, provided that each copy
contains the above copyright notice and this license. These can be
included either as stand-alone text files, human-readable headers or
in the appropriate machine-readable metadata fields within text or
binary files as long as those fields can be easily viewed by the user.

3) No Modified Version of the Font Software may use the Reserved Font
Name(s) unless explicit written permission is granted by the corresponding
Copyright Holder. This restriction only applies to the primary font name as
presented to the users.

4) The name(s) of the Copyright Holder(s) or the Author(s) of the Font
Software shall not be used to promote, endorse or advertise any
Modified Version, except to acknowledge the contribution(s) of the
Copyright Holder(s) and the Author(s) or with their explicit written
permission.

5) The Font Software, modified or unmodified, in part or in whole,
must be distributed entirely under this license, and must not be
distributed under any other license. The requirement for fonts to
remain under this license does not apply to any document created
using the Font Software.

TERMINATION
This license becomes null and void if any of the above conditions are
not met.

DISCLAIMER
THE FONT SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT
OF COPYRIGHT, PATENT, TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL THE
COPYRIGHT HOLDER BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
INCLUDING ANY GENERAL, SPECIAL, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL
DAMAGES, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF THE USE OR INABILITY TO USE THE FONT SOFTWARE OR FROM
OTHER DEALINGS IN THE FONT SOFTWARE.
```

### 6.5 ISC-licensed components (lucide-react)

```text
ISC License

Copyright (c) for portions of Lucide are held by Cole Bemis 2013-2022 as part of
Feather (MIT). All other copyright (c) for Lucide project and other contributors.

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND
FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THIS SOFTWARE.
```

---

**Contact:** nuxxzul@gmail.com
