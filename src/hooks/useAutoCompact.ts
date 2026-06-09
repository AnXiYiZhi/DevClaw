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
  boundaryRef?: RefObject<HTMLDivElement | null>,
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

    // Use the boundary element (e.g. parent with overflow-hidden) to measure
    // available space, and compare against the content element's scrollWidth.
    // This avoids issues where flex-1 + min-w-0 shrinks the observed element
    // itself rather than letting content overflow.
    const boundary = boundaryRef?.current ?? el;

    const checkOverflow = () => {
      const containerWidth = boundary.clientWidth;

      // Read the inner content element's natural width directly.
      // el.scrollWidth is unreliable in flex layouts: when the child fits,
      // ml-auto absorbs extra space and scrollWidth reports the container
      // width instead of the child's natural width.
      const child = el.firstElementChild as HTMLElement | null;
      if (!child) return;
      const contentWidth = child.scrollWidth;

      if (!compactRef.current) {
        // Overflow detected → switch to compact immediately (no lock)
        if (contentWidth > containerWidth + 1) {
          normalWidthRef.current = contentWidth;
          setCompact(true);
        }
      } else {
        // In compact mode: expand back when available space is clearly wider
        // than current compact content (1.3x).
        if (Date.now() < lockUntilRef.current) return;
        if (containerWidth > contentWidth * 1.3) {
          // Lock out resize events during the expand animation (200ms + 50ms margin)
          lockUntilRef.current = Date.now() + 250;
          setCompact(false);

          // After lock expires, re-check in case the container still overflows
          // (ResizeObserver won't fire if container size hasn't changed)
          if (checkTimeoutRef.current) clearTimeout(checkTimeoutRef.current);
          checkTimeoutRef.current = setTimeout(() => {
            const recheckChild = el.firstElementChild as HTMLElement | null;
            if (
              recheckChild &&
              recheckChild.scrollWidth > boundary.clientWidth + 1
            ) {
              normalWidthRef.current = recheckChild.scrollWidth;
              setCompact(true);
            }
          }, 300);
        }
      }
    };

    const ro = new ResizeObserver(checkOverflow);

    // Observe both the content element and the boundary
    ro.observe(el);
    ro.observe(boundary);

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
