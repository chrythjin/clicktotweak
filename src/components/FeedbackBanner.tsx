import React from "react";
import { AlertCircle, CheckCircle2 } from "lucide-react";

interface FeedbackBannerProps {
  message: string | null;
  isError?: boolean;
}

export const FeedbackBanner: React.FC<FeedbackBannerProps> = ({
  message,
  isError = false,
}) => {
  if (!message) return null;

  return (
    <div
      className={`p-3 rounded-xl border text-xs flex items-center gap-2 ${
        isError
          ? "bg-rose-950/40 border-rose-500/40 text-rose-300"
          : "bg-emerald-950/40 border-emerald-500/40 text-emerald-300"
      }`}
    >
      {isError ? (
        <AlertCircle className="w-4 h-4 text-rose-400 shrink-0" />
      ) : (
        <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
      )}
      <span>{message}</span>
    </div>
  );
};

