import { useEffect, useState } from "react";
import { FileText, Scale, ShieldCheck } from "lucide-react";
import { PageBody, PageHeader } from "@/components/layout/PageHeader";
import { Panel, PanelHeader, SectionLabel } from "@/components/ui/panel";
import { Segmented } from "@/components/ui/segmented";
import { Markdown } from "@/lib/markdown";
import { LEGAL_DOCS, LEGAL_UPDATED, LEGAL_VERSION } from "@/lib/legal";
import { useApp } from "@/stores/app";

type Tab = "about" | "license" | "privacy" | "disclaimer" | "thirdParty";

const TABS: { value: Tab; label: string }[] = [
  { value: "about", label: "About" },
  { value: "license", label: "Licence" },
  { value: "privacy", label: "Privacy" },
  { value: "disclaimer", label: "Disclaimer" },
  { value: "thirdParty", label: "Third-party" },
];

export default function Legal() {
  const version = useApp((state) => state.bootstrap?.version ?? "0.1.0");
  const [tab, setTab] = useState<Tab>("about");

  // Documents differ greatly in length; start each one at the top.
  useEffect(() => {
    document.getElementById("app-main")?.scrollTo({ top: 0, behavior: "auto" });
  }, [tab]);

  return (
    <>
      <PageHeader
        eyebrow="Legal & About"
        title="Licence, privacy and disclaimers"
        description="The documents below are the same ones published with the source code. They explain what FrameLab is licensed under, what it does with your data, and what it does not promise."
        actions={
          <Segmented
            value={tab}
            onChange={setTab}
            options={TABS}
            ariaLabel="Legal documents"
            className="flex-wrap"
          />
        }
      />
      <PageBody>
        {tab === "about" ? (
          <>
            <div className="grid gap-3 lg:grid-cols-3">
              <Panel>
                <PanelHeader title="FrameLab" hint={`Version ${version}`} action={<ShieldCheck className="size-3.5 text-text-3" />} />
                <div className="space-y-1.5 border-t border-line px-4 py-3 text-[12px] leading-relaxed text-text-2">
                  <p>A local-first performance monitor and benchmark recorder for Windows games.</p>
                  <p className="text-text-3">Copyright (c) 2026 FrameLab.</p>
                </div>
              </Panel>
              <Panel>
                <PanelHeader title="Licence" hint="MIT" action={<Scale className="size-3.5 text-text-3" />} />
                <div className="space-y-1.5 border-t border-line px-4 py-3 text-[12px] leading-relaxed text-text-2">
                  <p>
                    FrameLab is open source under the MIT License. You may use, modify and redistribute
                    it, provided the copyright and permission notice are kept.
                  </p>
                  <p className="text-text-3">The software is provided “as is”, without warranty.</p>
                </div>
              </Panel>
              <Panel>
                <PanelHeader title="Privacy" hint="Local only" action={<FileText className="size-3.5 text-text-3" />} />
                <div className="space-y-1.5 border-t border-line px-4 py-3 text-[12px] leading-relaxed text-text-2">
                  <p>No accounts, no telemetry, no uploads. Measurements stay on this computer.</p>
                  <p className="text-text-3">FrameLab's own code makes no outbound network requests.</p>
                </div>
              </Panel>
            </div>

            <SectionLabel>What these documents do and do not do</SectionLabel>
            <Panel>
              <div className="space-y-2 px-4 py-3 text-[12px] leading-relaxed text-text-2">
                <p>
                  The disclaimer describes the limits of what FrameLab promises. It does not, and cannot,
                  exclude rights that the law does not allow to be excluded — including applicable
                  consumer guarantees. No click, notice or acknowledgement removes those rights.
                </p>
                <p>
                  The privacy policy explains what is read from this machine and where it is stored. It is
                  a notice for your information, not a consent form for data collection, because no data
                  is collected by the developer.
                </p>
                <p className="text-text-3">
                  Documents last updated {LEGAL_UPDATED} · notice version {LEGAL_VERSION}
                </p>
              </div>
            </Panel>

            <SectionLabel>Third-party software</SectionLabel>
            <Panel>
              <PanelHeader
                title="Bundled components"
                hint="Intel PresentMon (MIT), the Inter typeface (SIL OFL 1.1), Tauri and open-source libraries."
                action={
                  <button
                    type="button"
                    onClick={() => setTab("thirdParty")}
                    className="text-[11.5px] text-accent hover:underline"
                  >
                    Read the notices
                  </button>
                }
              />
              <div className="border-t border-line px-4 py-2.5 text-[11.5px] leading-relaxed text-text-3">
                FrameLab is not affiliated with, endorsed by or sponsored by Intel, Microsoft or the
                authors of any bundled library. See the third-party tab for full licence texts.
              </div>
            </Panel>

            <div className="px-1 text-[10.5px] leading-relaxed text-text-3">
              These documents are provided for transparency and do not constitute legal advice.
            </div>
          </>
        ) : (
          <Panel>
            <div className="px-5 py-4">
              <Markdown text={LEGAL_DOCS[tab]} />
            </div>
          </Panel>
        )}
      </PageBody>
    </>
  );
}
