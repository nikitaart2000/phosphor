import { TerminalPanel } from '../shared/TerminalPanel';

interface HistoryRecord {
  id: number;
  source: string;
  target: string;
  uid: string;
  date: string;
  status: 'ok' | 'fail';
}

const DEMO_HISTORY: HistoryRecord[] = [
  { id: 1, source: 'EM4100', target: 'T5577', uid: '1A2B3C4D5E', date: '2026-02-10 14:23', status: 'ok' },
  { id: 2, source: 'HID ProxII', target: 'T5577', uid: '00A1B2C3', date: '2026-02-10 14:18', status: 'ok' },
  { id: 3, source: 'Indala', target: 'T5577', uid: 'F0E1D2C3B4', date: '2026-02-10 13:55', status: 'fail' },
  { id: 4, source: 'EM4100', target: 'T5577', uid: '5A5B5C5D5E', date: '2026-02-09 17:42', status: 'ok' },
  { id: 5, source: 'AWID', target: 'T5577', uid: '0012345678', date: '2026-02-09 16:30', status: 'ok' },
];

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

export function HistoryView() {
  const tableLines = buildTable(DEMO_HISTORY);

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
      <div style={{ marginTop: '12px', fontSize: '11px', color: 'var(--green-dim)' }}>
        {DEMO_HISTORY.length} records | {DEMO_HISTORY.filter(r => r.status === 'ok').length} successful
      </div>
    </TerminalPanel>
  );
}
