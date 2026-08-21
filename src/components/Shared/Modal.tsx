import { type CSSProperties, type ReactNode } from "react";
import { useDialog, type UseDialogOptions } from "../../hooks/useDialog";
import { ModalCloseButton } from "../Layout/ModalCloseButton";

export interface ModalProps extends UseDialogOptions {
  children: ReactNode;
  title?: ReactNode;
  subtitle?: ReactNode;
  ariaLabel?: string;
  ariaLabelledBy?: string;
  maxWidth?: number | string;
  showCloseButton?: boolean;
  closeButtonLabel?: string;
  headerActions?: ReactNode;
  className?: string;
  style?: CSSProperties;
  backdropStyle?: CSSProperties;
}

export function Modal({
  isOpen = true,
  onClose,
  initialFocusRef,
  closeOnEscape = true,
  title,
  subtitle,
  ariaLabel,
  ariaLabelledBy,
  maxWidth = "min(680px, 95vw)",
  showCloseButton = true,
  closeButtonLabel = "Close modal",
  headerActions,
  className,
  style,
  backdropStyle,
  children,
}: ModalProps) {
  const { dialogRef, handleKeyDown } = useDialog<HTMLDivElement>({
    isOpen,
    onClose,
    initialFocusRef,
    closeOnEscape,
  });

  if (!isOpen) return null;

  const titleId = title ? "modal-dialog-title" : undefined;
  const effectiveLabelledBy = ariaLabelledBy || (ariaLabel ? undefined : titleId);

  return (
    <div
      role="presentation"
      onKeyDown={handleKeyDown}
      onMouseDown={(e) => {
        if (e.target === e.currentTarget && onClose) {
          onClose();
        }
      }}
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 12000,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: 24,
        background: "rgba(4,3,6,.72)",
        backdropFilter: "blur(5px)",
        ...backdropStyle,
      }}
    >
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-label={ariaLabel}
        aria-labelledby={effectiveLabelledBy}
        tabIndex={-1}
        className={className}
        style={{
          width: typeof maxWidth === "number" ? `${maxWidth}px` : maxWidth,
          maxHeight: "90vh",
          display: "flex",
          flexDirection: "column",
          borderRadius: 12,
          border: "1px solid var(--border-primary, #332d3e)",
          background: "var(--bg-secondary, #15111b)",
          boxShadow: "0 20px 70px rgba(0,0,0,.5)",
          color: "var(--text-primary)",
          overflow: "hidden",
          outline: "none",
          ...style,
        }}
      >
        {(title || showCloseButton || headerActions) && (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              padding: "16px 20px",
              borderBottom: "1px solid var(--border-secondary, #25202e)",
              flexShrink: 0,
              gap: 12,
            }}
          >
            <div style={{ minWidth: 0, flex: 1 }}>
              {title && (
                typeof title === "string" ? (
                  <h2
                    id={titleId}
                    style={{
                      margin: 0,
                      fontSize: "var(--text-lg)",
                      fontWeight: 700,
                      color: "var(--text-primary)",
                    }}
                  >
                    {title}
                  </h2>
                ) : (
                  <div id={titleId}>{title}</div>
                )
              )}
              {subtitle && (
                <div
                  style={{
                    fontSize: "var(--text-xs)",
                    color: "var(--text-tertiary)",
                    marginTop: 3,
                  }}
                >
                  {subtitle}
                </div>
              )}
            </div>

            <div style={{ display: "flex", alignItems: "center", gap: 8, flexShrink: 0 }}>
              {headerActions}
              {showCloseButton && onClose && (
                <ModalCloseButton label={closeButtonLabel} onClick={onClose} />
              )}
            </div>
          </div>
        )}

        <div
          style={{
            flex: 1,
            overflowY: "auto",
            padding: (title || showCloseButton || headerActions) ? "16px 20px" : undefined,
          }}
        >
          {children}
        </div>
      </div>
    </div>
  );
}
