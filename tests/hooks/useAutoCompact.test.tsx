import { useLayoutEffect, useRef } from "react";
import { act, render, screen, waitFor } from "@testing-library/react";
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

const resizeObservers = new Set<ResizeObserverCallback>();

const triggerResizeObservers = () => {
  resizeObservers.forEach((callback) => {
    callback([], {} as ResizeObserver);
  });
};

class TestResizeObserver {
  private readonly callback: ResizeObserverCallback;

  constructor(callback: ResizeObserverCallback) {
    this.callback = callback;
    resizeObservers.add(callback);
  }

  observe() {
    this.callback([], this as unknown as ResizeObserver);
  }

  unobserve() {}

  disconnect() {
    resizeObservers.delete(this.callback);
  }
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
    resizeObservers.clear();
    vi.unstubAllGlobals();
  });

  it("switches to compact when toolbar content exceeds available width", async () => {
    vi.stubGlobal("ResizeObserver", TestResizeObserver);

    render(<CompactHarness containerWidth={180} contentWidth={360} />);

    await waitFor(() => {
      expect(screen.getByTestId("compact-state")).toHaveTextContent("compact");
    });
  });

  it("stays normal when toolbar content fits", async () => {
    vi.stubGlobal("ResizeObserver", TestResizeObserver);

    render(<CompactHarness containerWidth={360} contentWidth={180} />);

    await waitFor(() => {
      expect(screen.getByTestId("compact-state")).toHaveTextContent("normal");
    });
  });

  it("recovers from compact when the normal toolbar width fits again", async () => {
    vi.stubGlobal("ResizeObserver", TestResizeObserver);

    const { rerender } = render(
      <CompactHarness containerWidth={180} contentWidth={360} />,
    );

    await waitFor(() => {
      expect(screen.getByTestId("compact-state")).toHaveTextContent("compact");
    });

    rerender(<CompactHarness containerWidth={420} contentWidth={360} />);

    act(() => {
      triggerResizeObservers();
    });

    await waitFor(() => {
      expect(screen.getByTestId("compact-state")).toHaveTextContent("normal");
    });
  });

  it("retries expansion instead of waiting for a stale normal width", async () => {
    vi.stubGlobal("ResizeObserver", TestResizeObserver);

    const { rerender } = render(
      <CompactHarness containerWidth={180} contentWidth={360} />,
    );

    await waitFor(() => {
      expect(screen.getByTestId("compact-state")).toHaveTextContent("compact");
    });

    rerender(<CompactHarness containerWidth={340} contentWidth={320} />);

    act(() => {
      triggerResizeObservers();
    });

    await waitFor(() => {
      expect(screen.getByTestId("compact-state")).toHaveTextContent("normal");
    });
  });
});
