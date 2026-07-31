interface ModalCloseButtonProps {
  label: string;
  onClick: () => void;
}

export function ModalCloseButton({ label, onClick }: ModalCloseButtonProps) {
  return (
    <button type="button" className="modal-close-button" aria-label={label} onClick={onClick}>
      <i className="material-symbols-outlined" aria-hidden="true">close</i>
    </button>
  );
}
