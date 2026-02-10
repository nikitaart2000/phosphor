import { useState, useEffect } from 'react';
import { TerminalPanel } from '../shared/TerminalPanel';
import { useSfx } from '../../hooks/useSfx';

interface ConnectStepProps {
  onConnected: () => void;
}

export function ConnectStep({ onConnected }: ConnectStepProps) {
  const sfx = useSfx();
  const [detecting, setDetecting] = useState(true);
  const [dots, setDots] = useState('');
  const [deviceInfo, setDeviceInfo] = useState<{
    model: string;
    port: string;
    firmware: string;
  } | null>(null);

  // Animated dots
  useEffect(() => {
    if (!detecting) return;
    const timer = setInterval(() => {
      setDots(prev => (prev.length >= 3 ? '' : prev + '.'));
    }, 400);
    return () => clearInterval(timer);
  }, [detecting]);

  // Simulate device detection after 2s
  useEffect(() => {
    const timer = setTimeout(() => {
      setDetecting(false);
      setDeviceInfo({
        model: 'Proxmark3 Easy',
        port: 'COM3',
        firmware: 'Iceman v4.18',
      });
    }, 2000);
    return () => clearTimeout(timer);
  }, []);

  return (
    <TerminalPanel title="DEVICE">
      {detecting ? (
        <div style={{ color: 'var(--amber)', fontSize: '14px' }}>
          DETECTING DEVICE{dots}
        </div>
      ) : deviceInfo ? (
        <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
          <div style={{ color: 'var(--green-mid)' }}>
            [+] Device found
          </div>
          <div style={{ color: 'var(--green-dim)', marginTop: '8px' }}>
            MODEL : {deviceInfo.model}
          </div>
          <div style={{ color: 'var(--green-dim)' }}>
            PORT  : {deviceInfo.port}
          </div>
          <div style={{ color: 'var(--green-dim)' }}>
            FW    : {deviceInfo.firmware}
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
              CONNECT
            </button>
          </div>
        </div>
      ) : null}
    </TerminalPanel>
  );
}
