import { useEffect, useRef, type RefObject } from "react";

export interface UseDialogOptions {
  isOpen?: boolean;
  onClose?: () => void;
  initialFocusRef?: RefObject<HTMLElement | null>;
  closeOnEscape?: boolean;
}

const FOCUSABLE_SELECTOR =
  'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export function useDialog<T extends HTMLElement = HTMLDivElement>(
  options: UseDialogOptions = {}
): {
  dialogRef: RefObject<T | null>;
  handleKeyDown: (event: React.KeyboardEvent) => void;
} {
  const {
    isOpen = true,
    onClose,
    initialFocusRef,
    closeOnEscape = true,
  } = options;

  const dialogRef = useRef<T | null>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (!isOpen) return;

    // Record previous focus to restore upon closing
    previousFocusRef.current =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;

    // Focus initial element or first focusable / dialog container
    const dialog = dialogRef.current;
    if (initialFocusRef?.current) {
      initialFocusRef.current.focus();
    } else if (dialog) {
      const safeAction = dialog.querySelector<HTMLElement>("[data-safe-action]");
      if (safeAction) {
        safeAction.focus();
      } else {
        const focusables = dialog.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR);
        if (focusables.length > 0) {
          focusables[0]?.focus();
        } else {
          dialog.focus();
        }
      }
    }

    const handleWindowKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape" && closeOnEscape && onClose) {
        event.stopPropagation();
        onClose();
        return;
      }

      if (event.key === "Tab" && dialogRef.current) {
        const focusables = Array.from(
          dialogRef.current.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR)
        );
        if (focusables.length === 0) {
          event.preventDefault();
          return;
        }

        const first = focusables[0];
        const last = focusables[focusables.length - 1];
        const activeElement = document.activeElement;

        if (!dialogRef.current.contains(activeElement)) {
          event.preventDefault();
          (event.shiftKey ? last : first)?.focus();
        } else if (event.shiftKey && activeElement === first) {
          event.preventDefault();
          last?.focus();
        } else if (!event.shiftKey && activeElement === last) {
          event.preventDefault();
          first?.focus();
        }
      }
    };

    window.addEventListener("keydown", handleWindowKeyDown);

    return () => {
      window.removeEventListener("keydown", handleWindowKeyDown);
      previousFocusRef.current?.focus();
    };
  }, [isOpen, onClose, initialFocusRef, closeOnEscape]);

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === "Escape" && closeOnEscape && onClose) {
      event.stopPropagation();
      onClose();
    }
  };

  return { dialogRef, handleKeyDown };
}
