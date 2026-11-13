"use client";

import { useEffect, useRef } from "react";

type TurnstileApi = {
  render: (container: HTMLElement, options: { sitekey: string; action: string; callback: (token: string) => void; "expired-callback": () => void; "error-callback": () => void }) => string;
  remove: (widgetId: string) => void;
  reset: (widgetId: string) => void;
};

declare global { interface Window { turnstile?: TurnstileApi } }

type Props = { action: "auspice_genesis" | "auspice_signal"; disabled?: boolean; onToken: (token: string | null) => void };

/** Browser-side challenge only. Its token is never trusted by this component;
 * it is sent to the pinned OutLayer verifier for Siteverify validation. */
export function Turnstile({ action, disabled = false, onToken }: Props) {
  const container = useRef<HTMLDivElement>(null);
  const widgetId = useRef<string | null>(null);
  const siteKey = process.env.NEXT_PUBLIC_TURNSTILE_SITE_KEY;

  useEffect(() => {
    if (!siteKey || !container.current || disabled) return;
    const render = () => {
      if (!window.turnstile || !container.current || widgetId.current) return;
      widgetId.current = window.turnstile.render(container.current, {
        sitekey: siteKey,
        action,
        callback: (token) => onToken(token),
        "expired-callback": () => onToken(null),
        "error-callback": () => onToken(null),
      });
    };
    const knownScript = document.querySelector<HTMLScriptElement>('script[data-auspice-turnstile]');
    if (knownScript) {
      knownScript.addEventListener("load", render);
      render();
      return () => { knownScript.removeEventListener("load", render); if (widgetId.current && window.turnstile) window.turnstile.remove(widgetId.current); widgetId.current = null; };
    }
    const script = document.createElement("script");
    script.src = "https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit";
    script.async = true;
    script.defer = true;
    script.dataset.auspiceTurnstile = "true";
    script.addEventListener("load", render);
    document.head.appendChild(script);
    return () => { script.removeEventListener("load", render); if (widgetId.current && window.turnstile) window.turnstile.remove(widgetId.current); widgetId.current = null; };
  }, [action, disabled, onToken, siteKey]);

  if (!siteKey) return <p className="notice">Human verification is not configured for this launch.</p>;
  return <div aria-label="Human verification challenge" ref={container} />;
}
