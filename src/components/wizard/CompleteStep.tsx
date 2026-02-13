import { TerminalPanel } from '../shared/TerminalPanel';
import { useSfx } from '../../hooks/useSfx';
import type { CardType, CardData } from '../../machines/types';

interface CompleteStepProps {
  onReset: () => void;
  onDisconnect?: () => void;
  cardType?: CardType | null;
  cardData?: CardData | null;
  timestamp?: string | null;
}

export function CompleteStep({ onReset, onDisconnect, cardType, cardData, timestamp }: CompleteStepProps) {
  const sfx = useSfx();

  const displayType = cardType || 'Unknown';
  const displayUid = cardData?.uid || 'N/A';
  const displayTime = (() => {
    const iso = timestamp || new Date().toISOString();
    const d = new Date(iso);
    if (isNaN(d.getTime())) return iso.replace('T', ' ').slice(0, 19);
    const pad = (n: number) => String(n).padStart(2, '0');
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
  })();

  return (
    <TerminalPanel title="COMPLETE">
      <div style={{ fontSize: '13px', lineHeight: '1.8' }}>
        <div style={{ color: 'var(--green-bright)', fontWeight: 700, marginBottom: '12px' }}>
          [OK] OPERATION COMPLETE
        </div>

        <div style={{ color: 'var(--green-dim)' }}>
          SOURCE : {displayType} / {displayUid}
        </div>
        <div style={{ color: 'var(--green-dim)' }}>
          TARGET : Clone (verified)
        </div>
        <div style={{ color: 'var(--green-dim)' }}>
          TIME   : {displayTime}
        </div>
        <div style={{ color: 'var(--green-dim)' }}>
          STATUS : VERIFIED
        </div>

        <div style={{ marginTop: '20px', display: 'flex', alignItems: 'center', gap: '16px' }}>
          <button
            onClick={() => { sfx.action(); onReset(); }}
            style={{
              background: 'var(--bg-void)',
              color: 'var(--green-bright)',
              border: '2px solid var(--green-bright)',
              fontFamily: 'var(--font-mono)',
              fontSize: '14px',
              fontWeight: 600,
              padding: '8px 24px',
              cursor: 'pointer',
            }}
            onMouseEnter={(e) => {
              sfx.hover();
              e.currentTarget.style.background = 'var(--green-ghost)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = 'var(--bg-void)';
            }}
          >
            CLONE ANOTHER
          </button>
          {onDisconnect && (
            <span
              onClick={() => { sfx.action(); onDisconnect(); }}
              style={{
                color: 'var(--green-dim)',
                fontSize: '12px',
                cursor: 'pointer',
                fontFamily: 'var(--font-mono)',
                userSelect: 'none',
              }}
              onMouseEnter={(e) => {
                sfx.hover();
                e.currentTarget.style.color = 'var(--green-bright)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.color = 'var(--green-dim)';
              }}
            >
              [DISCONNECT]
            </span>
          )}
        </div>
      </div>
    </TerminalPanel>
  );
}
