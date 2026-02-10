import { useState, useEffect, useRef } from 'react';

export function useTypewriter(text: string, speed: number = 20): string {
  const [displayed, setDisplayed] = useState('');
  const indexRef = useRef(0);
  const prevTextRef = useRef(text);

  useEffect(() => {
    // Reset when text changes
    if (prevTextRef.current !== text) {
      prevTextRef.current = text;
      indexRef.current = 0;
      setDisplayed('');
    }

    if (indexRef.current >= text.length) return;

    const timer = setInterval(() => {
      indexRef.current++;
      setDisplayed(text.slice(0, indexRef.current));

      if (indexRef.current >= text.length) {
        clearInterval(timer);
      }
    }, speed);

    return () => clearInterval(timer);
  }, [text, speed]);

  return displayed;
}
