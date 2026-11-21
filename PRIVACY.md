# Privacy Policy

**FrameLab — how the application handles data**

Last updated: 21 November 2026

Applies to: FrameLab 0.1.0 and later development builds

FrameLab is built so that performance monitoring does not require a cloud service. This policy
explains what the application reads, where it is stored, and what leaves your computer. It is written
to align with the Australian Privacy Principles in the *Privacy Act 1988* (Cth) where they apply.

**Summary:** FrameLab does not collect, transmit, sell or share your personal information. Everything
it measures is processed and stored on your own computer.

---

## 1. Who is responsible

FrameLab is developed and distributed by an individual developer based in Australia ("the developer",
"we", "us"). For privacy questions, contact **nuxxzul@gmail.com**.

Because FrameLab does not send data to the developer, the developer does not receive or hold your
monitoring data, session recordings or hardware details.

## 2. What FrameLab reads on your computer

To display and record performance information, FrameLab reads the following from the local system:

| Data | Why it is read |
| --- | --- |
| Hardware identifiers and capabilities — CPU model, core and thread counts, GPU model, installed memory, display resolution and refresh rate, operating system version | To display your system and attach context to a recorded session |
| Performance counters — CPU utilisation and clock, per-core load, memory use, GPU engine utilisation, VRAM use, disk and network throughput | To show live monitoring and to store samples alongside frames |
| Process information — process names, process IDs and, where the operating system exposes it, executable paths | To let you choose which running game to monitor |
| Frame timing events — presentation and display timestamps for the process you attach to | To compute frame times, FPS statistics and stutters |
| Game library metadata — titles and install paths read from local Steam and Epic manifests | To build your games library |
| Application settings and recording metadata | To remember your preferences and describe each session |

This information is used **only** to provide the application's features on your device. It is not used
for advertising, profiling, analytics or any secondary purpose.

FrameLab does not read document contents, browsing history, keystrokes, clipboard contents, screen
images, audio, credentials or files unrelated to the features above.

## 3. What leaves your computer

**FrameLab's own code makes no outbound network connections.** It has no telemetry, analytics, crash
reporting, advertising, update pings or licence checks. Monitoring, capture, recording, analysis and
export all work offline.

Two things outside FrameLab's own code can involve the network:

1. **Microsoft Edge WebView2.** FrameLab's interface is rendered by the WebView2 runtime. WebView2 is a
   Microsoft component and may contact Microsoft services according to Microsoft's own terms and
   privacy statement. FrameLab does not control that behaviour.
2. **Installer runtime download.** If the WebView2 runtime is missing, the Windows installer can
   download it from Microsoft. No information about your sessions is involved.

If you obtain FrameLab from GitHub, GitHub may collect information about your download or visit under
GitHub's privacy statement. If you email the developer, your email address and message are handled so
that the enquiry can be answered.

## 4. Cookies and similar technologies

FrameLab sets no cookies, uses no local storage for tracking, and contains no third-party analytics or
advertising SDKs.

## 5. Where your data is stored

All application data stays in your Windows user profile:

| Path | Contents |
| --- | --- |
| `%APPDATA%\app.framelab.desktop\framelab.db` | Recorded sessions: frame timings, system samples, statistics and session metadata |
| `%APPDATA%\app.framelab.desktop\settings.json` | Your preferences, including diagnostic thresholds and display options |
| `%LOCALAPPDATA%\app.framelab.desktop\` | WebView2 runtime data used to render the interface |
| A location you choose | CSV or JSON files you export |

Recorded sessions can contain the name and path of a game executable and the hardware configuration of
the machine. Treat exported files as you would any other file about your computer.

## 6. Retention and deletion

- Recordings remain until you delete them. You can delete a single session in **History** or remove all
  sessions in **Settings → Data**.
- An optional retention period can remove sessions older than a chosen number of days.
- Uninstalling FrameLab removes the application but **does not** delete your recorded sessions or
  settings. To remove them, delete `%APPDATA%\app.framelab.desktop` after uninstalling.
- Exported CSV or JSON files are outside the application's control; delete them yourself.

## 7. Security

FrameLab stores data as ordinary files in your user profile, protected by your Windows account and
normal file permissions. **Recordings and settings are not encrypted at rest.** Anyone with access to
your Windows account or an administrative backup of your profile can read them.

FrameLab verifies the bundled PresentMon helper against a pinned SHA-256 hash before running it, runs
no shell commands on your behalf, and does not accept arbitrary file paths for writing without
validation. No security measure is perfect, and no software can be guaranteed free of defects.

## 8. Children

FrameLab is a performance tool for PC gaming and is not directed to children. It is not intended for
use by children under 15.

## 9. Overseas disclosure

The developer does not transfer your monitoring data overseas because the developer never receives it.
If you contact the developer by email, your message may be processed by email providers located outside
Australia.

## 10. Your rights

Depending on where you live, you may have rights to access, correct or delete personal information
about you, to object to processing, or to complain to a regulator.

- **Australia:** you may ask for access to, or correction of, personal information the developer holds
  about you (for example, an email you sent). If you believe your privacy has been interfered with,
  contact **nuxxzul@gmail.com** first; you may also complain to the Office of the Australian Information
  Commissioner at [oaic.gov.au](https://www.oaic.gov.au).
- **EU/EEA and UK:** because FrameLab does not transmit personal data to the developer, the developer
  does not act as a controller of your on-device monitoring data. You can still contact
  **nuxxzul@gmail.com** about any personal data you have sent directly.

The developer will respond to privacy enquiries within a reasonable period, normally within 30 days.

## 11. Changes to this policy

This policy may be updated as FrameLab changes. The current version is published in the repository and
is included in the application. Material changes will be reflected in the "last updated" date.

## 12. Not legal advice

This policy is provided for transparency. It is not legal advice, and it does not create rights beyond
those provided by applicable law.

---

**Contact:** nuxxzul@gmail.com
