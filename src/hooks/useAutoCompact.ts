import { useEffect, useRef, useState, type RefObject } from "react";

/**
 * Detects whether the container's children overflow the available width
 * and returns a `compact` flag for the AppSwitcher.
 *
 * Uses ResizeObserver on a flex-constrained container. The container
 * must have `flex-1 min-w-0 overflow-hidden` so its width is determined
 * by the parent layout, not its own content — avoiding the oscillation
 * problem when toggling compact mode.
 */
export function useAutoCompact(
  containerRef: RefObject<HTMLDivElement | null>,
): boolean {
  const [compact, setCompact] = useState(false);
  const compactRef = useRef(false);
  const normalWidthRef = useRef(0);
  const lockUntilRef = useRef(0);
  const checkTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Keep ref in sync with state so the observer callback always reads latest
  useEffect(() => {
    compactRef.current = compact;
  }, [compact]);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const checkOverflow = () => {
      if (Date.now() < lockUntilRef.current) return;

      if (!compactRef.current) {
        // Overflow detected → switch to compact
        if (el.scrollWidth > el.clientWidth + 1) {
          // Cache only at the overflow edge: when content fits,
          // scrollWidth === clientWidth (DOM spec), so caching unconditionally
          // would pollute normalWidthRef with the container width (e.g. after
          // maximizing), making the expand threshold unreachable.
          normalWidthRef.current = el.scrollWidth;
          setCompact(true);
        }
      } else if (normalWidthRef.current > 0) {
        // In compact mode: only recover to normal if
        // available space >= what normal mode needed
        if (el.clientWidth >= normalWidthRef.current) {
          // Lock out resize events during the expand animation (200ms + 50ms margin)
          lockUntilRef.current = Date.now() + 250;
          setCompact(false);

          // After lock expires, re-check in case the container still overflows
          // (ResizeObserver won't fire if container size hasn't changed)
          if (checkTimeoutRef.current) clearTimeout(checkTimeoutRef.current);
          checkTimeoutRef.current = setTimeout(() => {
            if (el.scrollWidth > el.clientWidth + 1) {
              normalWidthRef.current = el.scrollWidth;
              setCompact(true);
            }
          }, 300);
        }
      }
    };

    const ro = new ResizeObserver(checkOverflow);

    // Observe the container itself (detects window resize affecting this element)
    ro.observe(el);

    // Also observe the inner content wrapper so content size changes
    // (e.g. compact→normal text transition) also trigger overflow checks
    const inner = el.firstElementChild;
    if (inner) ro.observe(inner);

    return () => {
      ro.disconnect();
      if (checkTimeoutRef.current) clearTimeout(checkTimeoutRef.current);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return compact;
}
