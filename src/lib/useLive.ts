import { useEffect } from "react";
import { useApp } from "@/stores/app";

/// Polls the frame ring buffers while a page needs them. The sampling thread
/// keeps running in the backend; this only controls how often the UI reads.
export function useLiveFrames(active = true, intervalMs = 500) {
  const pollFrames = useApp((state) => state.pollFrames);
  useEffect(() => {
    if (!active) return;
    void pollFrames();
    const id = window.setInterval(() => void pollFrames(), intervalMs);
    return () => window.clearInterval(id);
  }, [active, intervalMs, pollFrames]);
}
