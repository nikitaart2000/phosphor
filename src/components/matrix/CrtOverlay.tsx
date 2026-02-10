export function CrtOverlay() {
  return (
    <div
      style={{
        pointerEvents: 'none',
        position: 'fixed',
        inset: 0,
        background:
          'repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(0,0,0,0.15) 2px, rgba(0,0,0,0.15) 4px)',
        zIndex: 9999,
        opacity: 0.35,
      }}
    />
  );
}
