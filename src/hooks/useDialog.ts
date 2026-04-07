import { useState, useCallback } from "react";

export function useDialog<T = void>() {
  const [open, setOpen] = useState(false);
  const [editing, setEditing] = useState<T | null>(null);

  const show = useCallback((data?: T) => {
    setEditing(data ?? null);
    setOpen(true);
  }, []);

  const close = useCallback(() => {
    setOpen(false);
    setEditing(null);
  }, []);

  const onOpenChange = useCallback((isOpen: boolean) => {
    if (!isOpen) {
      setOpen(false);
      setEditing(null);
    }
  }, []);

  return { open, editing, isEditing: editing !== null, show, close, onOpenChange };
}
