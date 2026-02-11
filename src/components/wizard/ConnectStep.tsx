import { useState, useEffect } from 'react';
import { TerminalPanel } from '../shared/TerminalPanel';
import { useSfx } from '../../hooks/useSfx';

interface ConnectStepProps {
  onConnected: () => void;
  isLoading?: boolean;
  device?: { model: string; port: string; firmware: string };
}

export function ConnectStep({ onConnected, isLoading, device }: ConnectStepProps) {
  const sfx = useSfx();
  const [dots, setDots] = useState('');

  // Animated dots while loading
  useEffect(() => {
    if (!isLoading) return;
    const timer = setInterval(() => {
      setDots(prev => (prev.length >= 3 ? '' : prev + '.'));
    }, 400);
    return () => clearInterval(timer);
  }, [isLoading]);

  return (
    <TerminalPanel title="DEVICE">
      {isLoading ? (
        <div style={{ color: 'var(--amber)', fontSize: '14px' }}>
          DETECTING DEVICE{dots}
        </div>
      ) : device ? (
        <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
          <div style={{ color: 'var(--green-mid)' }}>
            [+] Device found
          </div>
          <div style={{ color: 'var(--green-dim)', marginTop: '8px' }}>
            MODEL : {device.model}
          </div>
          <div style={{ color: 'var(--green-dim)' }}>
            PORT  : {device.port}
          </div>
          <div style={{ color: 'var(--green-dim)' }}>
            FW    : {device.firmware}
          </div>
          <div style={{ marginTop: '16px' }}>
            <button
              onClick={() => { sfx.action(); onConnected(); }}
              style={{
                background: 'var(--bg-void)',
                color: 'var(--green-bright)',
                border: '2px solid var(--green-bright)',
                fontFamily: 'var(--font-mono)',
                fontSize: '14px',
                fontWeight: 600,
                padding: '8px 24px',
                cursor: 'pointer',
                textTransform: 'uppercase',
              }}
              onMouseEnter={(e) => {
                sfx.hover();
                e.currentTarget.style.background = 'var(--green-ghost)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = 'var(--bg-void)';
              }}
            >
              SCAN
            </button>
          </div>
        </div>
      ) : (
        // Idle state: show connect button
        <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
          <div style={{ color: 'var(--green-dim)', marginBottom: '12px' }}>
            No device detected. Connect a Proxmark3 and press CONNECT.
          </div>
          <button
            onClick={() => { if (!isLoading) { sfx.action(); onConnected(); } }}
            disabled={isLoading}
            style={{
              background: 'var(--bg-void)',
              color: isLoading ? 'var(--green-dim)' : 'var(--green-bright)',
              border: `2px solid ${isLoading ? 'var(--green-dim)' : 'var(--green-bright)'}`,
              fontFamily: 'var(--font-mono)',
              fontSize: '14px',
              fontWeight: 600,
              padding: '8px 24px',
              cursor: isLoading ? 'not-allowed' : 'pointer',
              textTransform: 'uppercase',
              opacity: isLoading ? 0.5 : 1,
            }}
            onMouseEnter={(e) => {
              if (!isLoading) {
                sfx.hover();
                e.currentTarget.style.background = 'var(--green-ghost)';
              }
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'var(--bg-void)';
            }}
          >
            CONNECT
          </button>
        </div>
      )}
    </TerminalPanel>
  );
}
