// The in-app legal section renders the same documents that are published in
// the repository, imported verbatim so the two can never drift apart.
import licenseText from "../../LICENSE?raw";
import privacyText from "../../PRIVACY.md?raw";
import disclaimerText from "../../DISCLAIMER.md?raw";
import thirdPartyText from "../../THIRD_PARTY_NOTICES.md?raw";

// Bump whenever the substance of the notices changes, so existing users are
// shown the updated notice once more.
export const LEGAL_VERSION = "2026-11-21";

export const LEGAL_DOCS = {
  license: licenseText,
  privacy: privacyText,
  disclaimer: disclaimerText,
  thirdParty: thirdPartyText,
} as const;

export type LegalDocKey = keyof typeof LEGAL_DOCS;

export const LEGAL_UPDATED = "21 November 2026";
