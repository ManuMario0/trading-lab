/**
 * useProcessMonitor.ts
 * 
 * A specialized hook for polling the backend Orchestrator API.
 * 
 * Responsibilities:
 * - Fetching the list of active processes (PID, Name)
 * - Maintaining 'lastUpdated' state for polling feedback
 * - Providing 'stopProcess' functionality
 */
import { useCallback, useEffect, useMemo, useState } from 'react';
import type { ProcessInfo } from '../../../api/client';
import { apiClient } from '../../../api/client';

export type ProcessStatus = ProcessInfo;

export function useProcessMonitor() {
    const [processes, setProcesses] = useState<ProcessStatus[]>([]);
    const [lastUpdated, setLastUpdated] = useState<Date>(new Date());
    const [error, setError] = useState<string | null>(null);

    const refresh = useCallback(async () => {
        try {
            const list = await apiClient.getProcesses();
            setProcesses(list);
            setError(null);
            setLastUpdated(new Date());
        } catch (e) {
            console.error("Failed to fetch processes", e);
            setError("Failed to connect to Orchestrator");
        }
    }, []);

    // Polling Effect
    useEffect(() => {
        refresh();
        const interval = setInterval(refresh, 2000);
        return () => clearInterval(interval);
    }, [refresh]);

    const actions = useMemo(() => ({
        refresh
    }), [refresh]);

    return {
        processes,
        lastUpdated,
        error,
        actions
    };
}
