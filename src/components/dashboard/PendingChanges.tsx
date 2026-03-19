import { useSyncStore } from "../../stores/syncStore";
import { getChangeTypeColor, getChangeTypeSymbol, formatBytes } from "../../lib/utils";

export default function PendingChanges() {
  const { pendingChanges } = useSyncStore();

  return (
    <div className="card">
      <h3 className="text-sm font-medium text-text mb-3">
        Pending Changes
        {pendingChanges.length > 0 && (
          <span className="ml-2 badge-yellow">{pendingChanges.length}</span>
        )}
      </h3>
      {pendingChanges.length === 0 ? (
        <div className="text-sm text-text-dim text-center py-4">
          No pending changes
        </div>
      ) : (
        <div className="space-y-1 max-h-48 overflow-y-auto">
          {pendingChanges.map((change) => (
            <div key={change.path} className="flex items-center gap-2 text-xs font-mono">
              <span className={`font-bold w-3 ${getChangeTypeColor(change.change_type)}`}>
                {getChangeTypeSymbol(change.change_type)}
              </span>
              <span className="text-text-muted flex-1 truncate">{change.path}</span>
              {change.size_bytes !== undefined && (
                <span className="text-text-dim">{formatBytes(change.size_bytes)}</span>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
