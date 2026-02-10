interface TopBarProps {
  connected: boolean;
}

export function TopBar({ connected }: TopBarProps) {
  return (
    <div
      style={{
        height: '32px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '0 12px',
        background: 'var(--bg-panel)',
        borderBottom: '1px solid var(--green-dim)',
        fontFamily: 'var(--font-mono)',
        fontSize: '13px',
        zIndex: 10,
        position: 'relative',
      }}
    >
      <div style={{ color: 'var(--green-mid)', fontWeight: 600 }}>
        {'|||'} PHOSPHOR v1.0 {'|||'}
      </div>
      <div
        style={{
          color: connected ? 'var(--green-bright)' : 'var(--red-bright)',
          fontWeight: 500,
        }}
      >
        {connected ? '[PM3:CONNECTED]' : '[PM3:DISCONNECTED]'}
      </div>
    </div>
  );
}
