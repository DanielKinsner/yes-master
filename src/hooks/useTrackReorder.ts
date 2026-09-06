import { useEffect, useRef, useState, type PointerEvent } from "react";

// Pointer capture works in the native WebView without competing with Tauri's
// OS file-drop handler. Track identity, rather than a stale row index, owns drag.
export function useTrackReorder(
  ids: string[],
  onReorder: (from: number, to: number) => void,
) {
  const [dragging, setDragging] = useState<string | null>(null);
  const [insertion, setInsertion] = useState<number | null>(null);
  const current = useRef({ ids, onReorder });
  current.current = { ids, onReorder };
  const drag = useRef<{
    id: string;
    y: number;
    startY: number;
    list: HTMLElement;
    active: boolean;
    boundary: number | null;
  } | null>(null);
  const frame = useRef(0);
  const clear = () => {
    cancelAnimationFrame(frame.current);
    drag.current = null;
    setDragging(null);
    setInsertion(null);
  };
  useEffect(() => () => cancelAnimationFrame(frame.current), []);
  const locate = () => {
    const d = drag.current;
    if (!d?.active) return;
    const rows = [...d.list.querySelectorAll<HTMLElement>("[data-reorder-id]")];
    d.boundary = rows.findIndex(
      (row) =>
        d.y <
        row.getBoundingClientRect().top +
          row.getBoundingClientRect().height / 2,
    );
    if (d.boundary < 0) d.boundary = rows.length;
    setInsertion(d.boundary);
  };
  const scroll = () => {
    const d = drag.current;
    if (!d) return;
    if (d.active) {
      const r = d.list.getBoundingClientRect();
      const speed =
        d.y < r.top + 36
          ? -Math.min(12, (r.top + 36 - d.y) / 3)
          : d.y > r.bottom - 36
            ? Math.min(12, (d.y - r.bottom + 36) / 3)
            : 0;
      if (speed) {
        d.list.scrollTop += speed;
        locate();
      }
    }
    frame.current = requestAnimationFrame(scroll);
  };
  return {
    dragging,
    insertion,
    handleProps: (id: string) => ({
      onPointerDown: (e: PointerEvent<HTMLElement>) => {
        if (e.button !== 0) return;
        const list = e.currentTarget.closest<HTMLElement>(
          "[data-reorder-list]",
        );
        if (!list) return;
        e.preventDefault();
        e.currentTarget.setPointerCapture(e.pointerId);
        drag.current = {
          id,
          y: e.clientY,
          startY: e.clientY,
          list,
          active: false,
          boundary: null,
        };
        frame.current = requestAnimationFrame(scroll);
      },
      onPointerMove: (e: PointerEvent<HTMLElement>) => {
        const d = drag.current;
        if (!d) return;
        d.y = e.clientY;
        if (!d.active && Math.abs(d.y - d.startY) > 4) {
          d.active = true;
          setDragging(d.id);
        }
        locate();
      },
      onPointerUp: () => {
        const d = drag.current;
        if (d?.active && d.boundary !== null) {
          const from = current.current.ids.indexOf(d.id);
          const to = Math.min(
            current.current.ids.length - 1,
            d.boundary - (from < d.boundary ? 1 : 0),
          );
          if (from >= 0 && to >= 0 && from !== to)
            current.current.onReorder(from, to);
        }
        clear();
      },
      onPointerCancel: clear,
      onLostPointerCapture: clear,
      onKeyDown: (e: React.KeyboardEvent<HTMLElement>) => {
        if (e.key === "Escape") clear();
        if (e.key !== "ArrowUp" && e.key !== "ArrowDown") return;
        e.preventDefault();
        e.stopPropagation();
        const from = current.current.ids.indexOf(id);
        const to = from + (e.key === "ArrowUp" ? -1 : 1);
        if (from >= 0 && to >= 0 && to < current.current.ids.length)
          current.current.onReorder(from, to);
      },
    }),
  };
}
