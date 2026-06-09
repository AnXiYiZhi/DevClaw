import { useEffect, useRef, useState, type RefObject } from "react";

/**
 * Detects whether the toolbar content overflows the available width.
 *
 * The observed element should be the flex-constrained toolbar itself
 * (`flex-1 min-w-0 overflow-hidden`). Measuring a wider parent can hide
 * overflow when fixed buttons sit next to the toolbar.
 */
export function useAutoCompact(
  containerRef: RefObject<HTMLDivElement | null>,
): boolean {
  const [compact, setCompact] = useState(false);
  const compactRef = useRef(false);
  const normalWidthRef = useRef(0);
  const lastFailedExpandWidthRef = useRef(0);
  const lockUntilRef = useRef(0);
  const checkTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const expandRetryStep = 48;

  const setCompactState = (nextCompact: boolean) => {
    compactRef.current = nextCompact;
    setCompact(nextCompact);
  };

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const getContentWidth = () => {
      const child = el.firstElementChild as HTMLElement | null;
      return Math.max(el.scrollWidth, child?.scrollWidth ?? 0);
    };

    const checkOverflow = () => {
      if (Date.now() < lockUntilRef.current) return;

      const containerWidth = el.clientWidth;
      const contentWidth = getContentWidth();

      if (!compactRef.current) {
        if (contentWidth > containerWidth + 1) {
          normalWidthRef.current = contentWidth;
          lastFailedExpandWidthRef.current = containerWidth;
          setCompactState(true);
        }
      } else if (normalWidthRef.current > 0) {
        if (
          containerWidth >= normalWidthRef.current ||
          containerWidth >= lastFailedExpandWidthRef.current + expandRetryStep
        ) {
          lockUntilRef.current = Date.now() + 250;
          setCompactState(false);

          if (checkTimeoutRef.current) clearTimeout(checkTimeoutRef.current);
          checkTimeoutRef.current = setTimeout(() => {
            if (getContentWidth() > el.clientWidth + 1) {
              normalWidthRef.current = getContentWidth();
              lastFailedExpandWidthRef.current = el.clientWidth;
              setCompactState(true);
            } else {
              lastFailedExpandWidthRef.current = 0;
            }
          }, 300);
        }
      }
    };

    const ro = new ResizeObserver(checkOverflow);
    ro.observe(el);

    const inner = el.firstElementChild;
    if (inner) ro.observe(inner);

    checkOverflow();

    return () => {
      ro.disconnect();
      if (checkTimeoutRef.current) clearTimeout(checkTimeoutRef.current);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return compact;
}
