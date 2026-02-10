import { useState, useEffect, useRef, useCallback } from 'react';
import musicSrc from '../assets/audio/phosphor.mp3';

const STORAGE_KEY = 'phosphor-music-enabled';

function getInitial(): boolean {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored === null ? true : stored === '1';
  } catch {
    return true;
  }
}

export function useMusic() {
  const [enabled, setEnabled] = useState(getInitial);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const startedRef = useRef(false);

  // Create audio element once
  useEffect(() => {
    const audio = new Audio(musicSrc);
    audio.loop = true;
    audio.volume = 0.3;
    audioRef.current = audio;
    return () => {
      audio.pause();
      audio.src = '';
    };
  }, []);

  // Start playback on first user interaction (browser autoplay policy)
  useEffect(() => {
    if (startedRef.current) return;

    const tryPlay = () => {
      if (!enabled || !audioRef.current || startedRef.current) return;
      audioRef.current.play().then(() => {
        startedRef.current = true;
      }).catch(() => {
        // Autoplay blocked — will retry on next interaction
      });
    };

    // Try immediately (works if user already interacted)
    tryPlay();

    // Also listen for first click/keydown
    const handler = () => {
      tryPlay();
      if (startedRef.current) {
        document.removeEventListener('click', handler);
        document.removeEventListener('keydown', handler);
      }
    };

    document.addEventListener('click', handler);
    document.addEventListener('keydown', handler);
    return () => {
      document.removeEventListener('click', handler);
      document.removeEventListener('keydown', handler);
    };
  }, [enabled]);

  // Sync play/pause with enabled state
  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;
    if (enabled) {
      audio.play().catch(() => {});
    } else {
      audio.pause();
    }
  }, [enabled]);

  const toggle = useCallback(() => {
    setEnabled(prev => {
      const next = !prev;
      try {
        localStorage.setItem(STORAGE_KEY, next ? '1' : '0');
      } catch {}
      return next;
    });
  }, []);

  return { enabled, toggle };
}
