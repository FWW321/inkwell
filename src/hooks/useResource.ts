import { useState, useEffect, useCallback } from "react";

export function useResource<T extends { id: string }>(
  loader: () => Promise<T[]>,
  deps: unknown[] = [],
) {
  const [items, setItems] = useState<T[]>([]);
  const [loading, setLoading] = useState(true);

  const reload = useCallback(async () => {
    try {
      setItems(await loader());
    } catch (err) {
      console.error("Failed to load resource:", err);
    } finally {
      setLoading(false);
    }
  }, deps);

  useEffect(() => {
    reload();
  }, [reload]);

  return {
    items,
    setItems,
    loading,
    reload,
    append: useCallback((item: T) => setItems((prev) => [...prev, item]), []),
    prepend: useCallback((item: T) => setItems((prev) => [item, ...prev]), []),
    replace: useCallback((id: string, item: T) => setItems((prev) => prev.map((i) => (i.id === id ? item : i))), []),
    remove: useCallback((id: string) => setItems((prev) => prev.filter((i) => i.id !== id)), []),
  };
}
