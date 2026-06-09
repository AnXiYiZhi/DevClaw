import { useLayoutEffect, useRef } from "react";
import { render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useAutoCompact } from "@/hooks/useAutoCompact";

const defineWidth = (
  element: HTMLElement,
  key: "clientWidth" | "scrollWidth",
  value: number,
) => {
  Object.defineProperty(element, key, {
    configurable: true,
    value,
  });
};

class ImmediateResizeObserver {
  private readonly callback: ResizeObserverCallback;

  constructor(callback: ResizeObserverCallback) {
    this.callback = callback;
  }

  observe() {
    this.callback([], this as unknown as ResizeObserver);
  }

  unobserve() {}

  disconnect() {}
}

const CompactHarness = ({
  containerWidth,
  contentWidth,
}: {
  containerWidth: number;
  contentWidth: number;
}) => {
  const toolbarRef = useRef<HTMLDivElement>(null);
  const innerRef = useRef<HTMLDivElement>(null);
  const compact = useAutoCompact(toolbarRef);

  useLayoutEffect(() => {
    const toolbar = toolbarRef.current;
    const inner = innerRef.current;
    if (!toolbar || !inner) return;

    defineWidth(toolbar, "clientWidth", containerWidth);
    defineWidth(toolbar, "scrollWidth", Math.max(containerWidth, contentWidth));
    defineWidth(inner, "scrollWidth", contentWidth);
  }, [containerWidth, contentWidth]);

  return (
    <div>
      <div data-testid="compact-state">{compact ? "compact" : "normal"}</div>
      <div ref={toolbarRef}>
        <div ref={innerRef} />
      </div>
    </div>
  );
};

describe("useAutoCompact", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("switches to compact when toolbar content exceeds available width", async () => {
    vi.stubGlobal("ResizeObserver", ImmediateResizeObserver);

    render(<CompactHarness containerWidth={180} contentWidth={360} />);

    await waitFor(() => {
      expect(screen.getByTestId("compact-state")).toHaveTextContent("compact");
    });
  });

  it("stays normal when toolbar content fits", async () => {
    vi.stubGlobal("ResizeObserver", ImmediateResizeObserver);

    render(<CompactHarness containerWidth={360} contentWidth={180} />);

    await waitFor(() => {
      expect(screen.getByTestId("compact-state")).toHaveTextContent("normal");
    });
  });
});
