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

// Box-drawing table
const H = '\u2500'; // horizontal
const V = '\u2502'; // vertical
const TL = '\u250C'; // top-left
const TR = '\u2510'; // top-right
const BL = '\u2514'; // bottom-left
const BR = '\u2518'; // bottom-right
const TJ = '\u252C'; // top-junction
const BJ = '\u2534'; // bottom-junction
const LJ = '\u251C'; // left-junction
const RJ = '\u2524'; // right-junction
const XJ = '\u253C'; // cross-junction

function pad(str: string, len: number): string {
  return str.padEnd(len).slice(0, len);
}

function buildTable(records: HistoryRecord[]): string[] {
  const cols = [
    { header: '#', width: 3 },
    { header: 'SOURCE', width: 10 },
    { header: 'TARGET', width: 8 },
    { header: 'UID', width: 12 },
    { header: 'DATE', width: 18 },
    { header: 'STATUS', width: 6 },
  ];

  const lines: string[] = [];

  // Top border
  lines.push(TL + cols.map(c => H.repeat(c.width + 2)).join(TJ) + TR);

  // Header
  lines.push(V + cols.map(c => ` ${pad(c.header, c.width)} `).join(V) + V);

  // Header separator
  lines.push(LJ + cols.map(c => H.repeat(c.width + 2)).join(XJ) + RJ);

  // Data rows
  for (const rec of records) {
    const statusStr = rec.status === 'ok' ? '[OK]' : '[!!]';
    const row = [
      pad(String(rec.id), cols[0].width),
      pad(rec.source, cols[1].width),
      pad(rec.target, cols[2].width),
      pad(rec.uid, cols[3].width),
      pad(rec.date, cols[4].width),
      pad(statusStr, cols[5].width),
    ];
    lines.push(V + row.map(cell => ` ${cell} `).join(V) + V);
  }

  // Bottom border
  lines.push(BL + cols.map(c => H.repeat(c.width + 2)).join(BJ) + BR);

  return lines;
}

/** Map backend CloneRecord to display HistoryRecord */
function toHistoryRecord(r: CloneRecord, index: number): HistoryRecord {
  return {
    id: index + 1,
    source: r.source_type,
    target: r.target_type,
    uid: r.source_uid || '---',
    date: r.timestamp,
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
        if (!cancelled) setLoading(false);
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

  const tableLines = buildTable(records);
  const successCount = records.filter(r => r.status === 'ok').length;

  return (
    <TerminalPanel title="CLONE HISTORY">
      <div
        style={{
          fontFamily: 'var(--font-mono)',
          fontSize: '12px',
          lineHeight: '1.5',
          whiteSpace: 'pre',
          overflowX: 'auto',
        }}
      >
        {tableLines.map((line, i) => (
          <div key={i} style={{ color: 'var(--green-mid)' }}>
            {line}
          </div>
        ))}
      </div>
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
