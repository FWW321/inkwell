import { useState, useRef, useCallback } from "react";

export interface StreamingState {
  isStreaming: boolean;
  streamingText: string;
  error: string | null;
}

export interface StreamingActions {
  start: () => void;
  finish: () => void;
  appendText: (text: string) => void;
  fail: (message: string) => void;
  reset: () => void;
}

export interface StreamingSession extends StreamingState, StreamingActions {
  readonly textRef: React.RefObject<string>;
}

export function useStreamingSession(): StreamingSession {
  const [isStreaming, setIsStreaming] = useState(false);
  const [streamingText, setStreamingText] = useState("");
  const [error, setError] = useState<string | null>(null);
  const textRef = useRef("");

  const start = useCallback(() => {
    textRef.current = "";
    setIsStreaming(true);
    setStreamingText("");
    setError(null);
  }, []);

  const appendText = useCallback((text: string) => {
    textRef.current += text;
    setStreamingText(textRef.current);
  }, []);

  const finish = useCallback(() => {
    setIsStreaming(false);
  }, []);

  const fail = useCallback((message: string) => {
    setIsStreaming(false);
    setError(message);
  }, []);

  const reset = useCallback(() => {
    textRef.current = "";
    setStreamingText("");
    setError(null);
    setIsStreaming(false);
  }, []);

  return { isStreaming, streamingText, error, textRef, start, finish, appendText, fail, reset };
}
