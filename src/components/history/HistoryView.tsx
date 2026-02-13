import { useEffect, useState } from 'react';
import { TerminalPanel } from '../shared/TerminalPanel';
import { getHistory } from '../../lib/api';
import type { CloneRecord } from '../../machines/types';

interface HistoryRecord {
  id: number;
  source: string;
  target: string;
  uid: string;
  date: string;
  status: 'ok' | 'fail';
}

function formatLocalTime(isoStr: string): string {
  const d = new Date(isoStr);
  if (isNaN(d.getTime())) return isoStr.replace('T', ' ').slice(0, 19);
  const pad2 = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())} ${pad2(d.getHours())}:${pad2(d.getMinutes())}:${pad2(d.getSeconds())}`;
}

/** Map backend CloneRecord to display HistoryRecord */
function toHistoryRecord(r: CloneRecord, index: number): HistoryRecord {
  return {
    id: index + 1,
    source: r.source_type,
    target: r.target_type,
    uid: r.source_uid || '---',
    date: formatLocalTime(r.timestamp),
    status: r.success ? 'ok' as const : 'fail' as const,
  };
}

export function HistoryView() {
  const [records, setRecords] = useState<HistoryRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    const start = Date.now();
    getHistory()
      .then((data: CloneRecord[]) => {
        if (!cancelled) {
          setRecords(data.map(toHistoryRecord));
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          const msg = err instanceof Error ? err.message : 'Failed to load history';
          setError(msg);
        }
      })
      .finally(() => {
        if (cancelled) return;
        const elapsed = Date.now() - start;
        const delay = Math.max(0, 300 - elapsed);
        setTimeout(() => { if (!cancelled) setLoading(false); }, delay);
      });
    return () => { cancelled = true; };
  }, [refreshKey]);

  const handleRefresh = () => setRefreshKey((k) => k + 1);

  // Loading state
  if (loading) {
    return (
      <TerminalPanel title="CLONE HISTORY">
        <div style={{
          fontFamily: 'var(--font-mono)',
          fontSize: '12px',
          color: 'var(--green-dim)',
          padding: '12px 0',
        }}>
          [..] Loading history...
        </div>
      </TerminalPanel>
    );
  }

  // Error state
  if (error) {
    return (
      <TerminalPanel title="CLONE HISTORY">
        <div style={{
          fontFamily: 'var(--font-mono)',
          fontSize: '12px',
          color: 'var(--red-bright)',
          padding: '12px 0',
        }}>
          [XX] Error loading history: {error}
        </div>
        <button
          onClick={handleRefresh}
          style={{
            background: 'none',
            border: '1px solid var(--green-dim)',
            color: 'var(--green-dim)',
            fontFamily: 'var(--font-mono)',
            fontSize: '11px',
            padding: '2px 8px',
            cursor: 'pointer',
          }}
          onMouseEnter={(e) => { e.currentTarget.style.color = 'var(--green-bright)'; e.currentTarget.style.borderColor = 'var(--green-bright)'; }}
          onMouseLeave={(e) => { e.currentTarget.style.color = 'var(--green-dim)'; e.currentTarget.style.borderColor = 'var(--green-dim)'; }}
        >
          RETRY
        </button>
      </TerminalPanel>
    );
  }

  // Empty state
  if (records.length === 0) {
    return (
      <TerminalPanel title="CLONE HISTORY">
        <div style={{
          fontFamily: 'var(--font-mono)',
          fontSize: '12px',
          color: 'var(--green-dim)',
          padding: '12px 0',
        }}>
          No clone history yet.
        </div>
      </TerminalPanel>
    );
  }

  const successCount = records.filter(r => r.status === 'ok').length;

  return (
    <TerminalPanel title="CLONE HISTORY">
      <table
        style={{
          width: '100%',
          borderCollapse: 'collapse',
          fontFamily: 'var(--font-mono)',
          fontSize: '12px',
        }}
      >
        <thead>
          <tr>
            {['#', 'SOURCE', 'TARGET', 'UID', 'DATE', 'STATUS'].map(h => (
              <th
                key={h}
                style={{
                  padding: '4px 8px',
                  textAlign: 'left',
                  color: 'var(--green-mid)',
                  borderBottom: '1px solid var(--green-dim)',
                  fontWeight: 600,
                  fontSize: '11px',
                  letterSpacing: '1px',
                }}
              >
                {h}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {records.map(rec => (
            <tr key={rec.id}>
              <td style={{ padding: '3px 8px', color: 'var(--green-mid)', borderBottom: '1px solid rgba(0,255,65,0.1)' }}>{rec.id}</td>
              <td style={{ padding: '3px 8px', color: 'var(--green-mid)', borderBottom: '1px solid rgba(0,255,65,0.1)' }}>{rec.source}</td>
              <td style={{ padding: '3px 8px', color: 'var(--green-mid)', borderBottom: '1px solid rgba(0,255,65,0.1)' }}>{rec.target}</td>
              <td style={{ padding: '3px 8px', color: 'var(--green-mid)', borderBottom: '1px solid rgba(0,255,65,0.1)' }}>{rec.uid}</td>
              <td style={{ padding: '3px 8px', color: 'var(--green-mid)', borderBottom: '1px solid rgba(0,255,65,0.1)' }}>{rec.date}</td>
              <td style={{ padding: '3px 8px', borderBottom: '1px solid rgba(0,255,65,0.1)', color: rec.status === 'ok' ? 'var(--green-bright)' : 'var(--red-bright)' }}>
                {rec.status === 'ok' ? '[OK]' : '[!!]'}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <div style={{ marginTop: '12px', fontSize: '11px', color: 'var(--green-dim)', display: 'flex', alignItems: 'center', gap: '12px' }}>
        <span>{records.length} records | {successCount} successful</span>
        <button
          onClick={handleRefresh}
          style={{
            background: 'none',
            border: '1px solid var(--green-dim)',
            color: 'var(--green-dim)',
            fontFamily: 'var(--font-mono)',
            fontSize: '11px',
            padding: '2px 8px',
            cursor: 'pointer',
          }}
          onMouseEnter={(e) => { e.currentTarget.style.color = 'var(--green-bright)'; e.currentTarget.style.borderColor = 'var(--green-bright)'; }}
          onMouseLeave={(e) => { e.currentTarget.style.color = 'var(--green-dim)'; e.currentTarget.style.borderColor = 'var(--green-dim)'; }}
        >
          REFRESH
        </button>
      </div>
    </TerminalPanel>
  );
}
