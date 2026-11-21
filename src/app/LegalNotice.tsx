import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Lock, Ruler, Scale } from "lucide-react";
import { Modal } from "@/components/ui/modal";
import { Button } from "@/components/ui/button";
import { LEGAL_VERSION } from "@/lib/legal";
import { useApp } from "@/stores/app";

/**
 * Shown once per notice version.
 *
 * This is a transparency notice, not a liability waiver: under Australian law a
 * click cannot exclude the consumer guarantees, and there is no data collection
 * to consent to. It exists so the privacy position, the accuracy limits and the
 * licence are put in front of the user before they rely on a measurement.
 */
export default function LegalNotice() {
  const settings = useApp((state) => state.settings);
  const patchSettings = useApp((state) => state.patchSettings);
  const [dismissed, setDismissed] = useState(false);
  const navigate = useNavigate();

  if (!settings || dismissed || settings.legalAcceptedVersion === LEGAL_VERSION) return null;

  const acknowledge = () => {
    void patchSettings({ legalAcceptedVersion: LEGAL_VERSION });
  };

  return (
    <Modal
      open
      onOpenChange={(open) => {
        if (!open) setDismissed(true);
      }}
      title="Before you start"
      description="A short notice about your data, the accuracy of the numbers, and the terms FrameLab is provided under."
      width={520}
      footer={
        <>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => {
              setDismissed(true);
              navigate("/legal");
            }}
          >
            Read the full documents
          </Button>
          <Button variant="primary" size="sm" onClick={acknowledge}>
            I understand
          </Button>
        </>
      }
    >
      <ul className="space-y-3 text-[12px] leading-relaxed text-text-2">
        <li className="flex gap-2.5">
          <Lock className="mt-0.5 size-3.5 shrink-0 text-text-3" />
          <span>
            <span className="text-text-1">Everything stays on this computer.</span> FrameLab has no
            accounts and no telemetry, and its own code makes no network requests. Sessions and settings
            are stored in your Windows profile until you delete them.
          </span>
        </li>
        <li className="flex gap-2.5">
          <Ruler className="mt-0.5 size-3.5 shrink-0 text-text-3" />
          <span>
            <span className="text-text-1">Measurements are estimates.</span> Figures come from Windows
            performance counters and frame-presentation events. They can be inaccurate, delayed or
            unavailable, and are not a diagnostic or safety instrument.
          </span>
        </li>
        <li className="flex gap-2.5">
          <Scale className="mt-0.5 size-3.5 shrink-0 text-text-3" />
          <span>
            <span className="text-text-1">Provided “as is” under the MIT licence.</span> Nothing here
            removes rights that cannot lawfully be excluded, including applicable consumer guarantees.
          </span>
        </li>
      </ul>
      <p className="mt-3 text-[10.5px] leading-relaxed text-text-3">
        You can revisit this summary at any time under Legal &amp; About. Notice version {LEGAL_VERSION}.
      </p>
    </Modal>
  );
}
