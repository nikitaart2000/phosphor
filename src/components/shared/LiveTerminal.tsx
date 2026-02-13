import { useRef, useEffect, useState } from 'react';
import { useTerminalLog, type LogLine } from '../../hooks/useTerminalLog';
import { useSettings } from '../../hooks/useSettings';
import { useWizard } from '../../hooks/useWizard';
import { runRawCommand } from '../../lib/api';
import { useSfx } from '../../hooks/useSfx';

const EXPANDED_HEIGHT = 200;

export function LiveTerminal() {
  const { lines, clear } = useTerminalLog();
  const { settings } = useSettings();
  const wizard = useWizard();
  const sfx = useSfx();
  const [collapsed, setCollapsed] = useState(false);
  const [maximized, setMaximized] = useState(false);
  const [cmdInput, setCmdInput] = useState('');
  const [sending, setSending] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const port = wizard.context.port;
  const expertMode = settings.expertMode;
  const canSend = !!port && !sending && cmdInput.trim().length > 0;

  // Auto-scroll to bottom on new lines
  useEffect(() => {
    if (scrollRef.current && !collapsed) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [lines, collapsed]);

  const toggle = () => {
    sfx.click();
    setCollapsed(prev => !prev);
  };

  const handleClear = () => {
    sfx.click();
    clear();
  };

  const handleSubmit = async () => {
    const cmd = cmdInput.trim();
    if (!cmd || !port || sending) return;
    sfx.action();
    setCmdInput('');
    setSending(true);
    try {
      await runRawCommand(port, cmd);
    } catch {
      // Output will appear via pm3-output event; errors are emitted by backend
    } finally {
      setSending(false);
      inputRef.current?.focus();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleSubmit();
    }
  };

  const arrow = collapsed ? '\u25B2' : '\u25BC';

  return (
    <div
      style={{
        fontFamily: 'var(--font-mono)',
        fontSize: '12px',
        color: 'var(--green-bright)',
        borderTop: '1px solid var(--green-dim)',
      }}
    >
      {/* Header bar */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          padding: '4px 8px',
          color: 'var(--green-dim)',
          cursor: 'pointer',
          userSelect: 'none',
          borderBottom: collapsed ? 'none' : '1px solid var(--green-dim)',
          boxShadow: collapsed ? 'none' : '0 1px 4px rgba(0,255,65,0.1)',
        }}
        onClick={toggle}
      >
        <span style={{ fontSize: '11px', letterSpacing: '2px', fontWeight: 600 }}>OUTPUT</span>
        <span style={{ flex: 1 }} />
        {/* CLR button */}
        <span
          onClick={(e) => { e.stopPropagation(); handleClear(); }}
          onMouseEnter={(e) => { sfx.hover(); e.currentTarget.style.color = 'var(--green-bright)'; }}
          onMouseLeave={(e) => { e.currentTarget.style.color = 'var(--green-dim)'; }}
          style={{ cursor: 'pointer', fontSize: '11px' }}
        >
          [CLR]
        </span>
        {/* Collapse toggle */}
        <span style={{ fontSize: '11px' }}>
          [{arrow}]
        </span>
        {/* Expand/maximize toggle */}
        <span
          onClick={(e) => { e.stopPropagation(); sfx.click(); setMaximized(prev => !prev); }}
          onMouseEnter={(e) => {
            sfx.hover();
            e.currentTarget.style.color = 'var(--green-bright)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.color = 'var(--green-dim)';
          }}
          style={{ cursor: 'pointer', fontSize: '11px' }}
        >
          {maximized ? '[\u2921]' : '[\u2922]'}
        </span>
      </div>

      {/* Log area */}
      {!collapsed && (
        <div
          ref={scrollRef}
          style={{
            height: `${maximized ? 500 : EXPANDED_HEIGHT}px`,
            overflowY: 'auto',
            background: 'var(--bg-void)',
            padding: '4px 8px',
            borderLeft: '1px solid var(--green-dim)',
            borderRight: '1px solid var(--green-dim)',
            borderBottom: expertMode ? 'none' : '1px solid var(--green-dim)',
            lineHeight: '1.5',
          }}
        >
          {lines.length === 0 ? (
            <div style={{ color: 'var(--green-dim)', fontStyle: 'italic' }}>
              Waiting for PM3 output...
            </div>
          ) : (
            lines.map((line, i) => (
              <TerminalLine key={i} line={line} />
            ))
          )}
        </div>
      )}

      {/* Expert mode command input */}
      {!collapsed && expertMode && (
        <ExpertInput
          port={port}
          cmdInput={cmdInput}
          setCmdInput={setCmdInput}
          canSend={canSend}
          sending={sending}
          inputRef={inputRef}
          onSubmit={handleSubmit}
          onKeyDown={handleKeyDown}
        />
      )}
    </div>
  );
}

function TerminalLine({ line }: { line: LogLine }) {
  return (
    <div style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>
      <span style={{ color: line.isError ? 'var(--red-bright)' : 'var(--green-mid)' }}>
        {line.text}
      </span>
    </div>
  );
}

interface ExpertInputProps {
  port: string | null;
  cmdInput: string;
  setCmdInput: (v: string) => void;
  canSend: boolean;
  sending: boolean;
  inputRef: React.RefObject<HTMLInputElement | null>;
  onSubmit: () => void;
  onKeyDown: (e: React.KeyboardEvent<HTMLInputElement>) => void;
}

function ExpertInput({ port, cmdInput, setCmdInput, canSend, sending, inputRef, onSubmit, onKeyDown }: ExpertInputProps) {
  const sfx = useSfx();
  const disabled = !port;

  return (
    <div style={{
      borderTop: '1px solid var(--green-dim)',
    }}>
      <div style={{
        padding: '2px 8px',
        fontSize: '11px',
        color: 'var(--green-dim)',
        letterSpacing: '2px',
        fontWeight: 600,
      }}>
        COMMAND
      </div>
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          padding: '4px 12px 6px',
          background: 'var(--bg-void)',
        }}
      >
        <span style={{ color: 'var(--green-mid)', whiteSpace: 'pre', fontSize: '12px' }}>
          pm3 &gt;
        </span>
        <input
          ref={inputRef}
          type="text"
          value={cmdInput}
          onChange={(e) => setCmdInput(e.target.value)}
          onKeyDown={onKeyDown}
          disabled={disabled}
          placeholder={disabled ? 'Connect device first' : 'Enter PM3 command...'}
          style={{
            flex: 1,
            background: 'transparent',
            border: 'none',
            outline: 'none',
            color: disabled ? 'var(--green-dim)' : 'var(--green-bright)',
            fontFamily: 'var(--font-mono)',
            fontSize: '12px',
            padding: '2px 0',
            caretColor: 'var(--green-bright)',
          }}
        />
        <span
          onClick={() => {
            if (canSend) {
              sfx.click();
              onSubmit();
            }
          }}
          onMouseEnter={(e) => {
            if (canSend) {
              sfx.hover();
              e.currentTarget.style.color = 'var(--green-bright)';
            }
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.color = canSend ? 'var(--green-mid)' : 'var(--green-dim)';
          }}
          style={{
            color: canSend ? 'var(--green-mid)' : 'var(--green-dim)',
            cursor: canSend ? 'pointer' : 'default',
            userSelect: 'none',
            fontSize: '12px',
            fontWeight: 600,
            opacity: sending ? 0.5 : 1,
          }}
        >
          [&gt;&gt;]
        </span>
      </div>
    </div>
  );
}
