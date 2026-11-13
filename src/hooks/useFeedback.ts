import { useCallback, useEffect, useRef, useState } from "react";

export interface FeedbackState {
  message: string | null;
  isError: boolean;
  notify: (msg: string, err?: boolean) => void;
  clear: () => void;
}

export function useFeedback(durationMs = 4000): FeedbackState {
  const [message, setMessage] = useState<string | null>(null);
  const [isError, setIsError] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clear = useCallback(() => {
    if (timerRef.current !== null) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    setMessage(null);
  }, []);

  const notify = useCallback(
    (msg: string, err = false) => {
      if (timerRef.current !== null) {
        clearTimeout(timerRef.current);
      }
      setMessage(msg);
      setIsError(err);
      timerRef.current = setTimeout(() => {
        setMessage(null);
        timerRef.current = null;
      }, durationMs);
    },
    [durationMs]
  );

  useEffect(() => {
    return () => {
      if (timerRef.current !== null) {
        clearTimeout(timerRef.current);
      }
    };
  }, []);

  return { message, isError, notify, clear };
}

