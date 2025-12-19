import type { Edge, Node } from "reactflow";
import type { Portfolio } from "../features/orchestrator/components/panel/Panel";
import { fromSavedLayout, toSavedLayout } from "../features/orchestrator/utils/layoutPersistence";

export interface ProcessInfo {
    id: string; // The UUID Key
    status: string;
}

export interface SpawnRequest {
    name: string;
    cmd: string;
    args: string[];
}

const API_BASE = "http://localhost:3000";

export const apiClient = {
    async getProcesses(): Promise<ProcessInfo[]> {
        const res = await fetch(`${API_BASE}/processes`);
        if (!res.ok) throw new Error("Failed to fetch processes");
        const data = await res.json();
        return data.processes;
    },

    async saveLayout(layoutId: string, nodes: Node[], edges: Edge[]): Promise<void> {
        const res = await fetch(`${API_BASE}/layout/${layoutId}`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(toSavedLayout(layoutId, nodes, edges)),
        });
        if (!res.ok) {
            const err = await res.json();
            throw new Error(err.msg || "Failed to save layout");
        }
    },

    async removeLayout(layoutId: string): Promise<void> {
        const res = await fetch(`${API_BASE}/layout/${layoutId}`, {
            method: "DELETE",
        });
        if (!res.ok) {
            const err = await res.json();
            throw new Error(err.msg || "Failed to remove layout");
        }
    },

    async getLayout(layoutId: string): Promise<{ nodes: Node[], edges: Edge[] }> {
        const res = await fetch(`${API_BASE}/layout/${layoutId}`);
        if (!res.ok) throw new Error("Failed to fetch layout");
        const response = await res.json();
        return fromSavedLayout(response.layout);
    },

    async deployLayout(layoutId: string): Promise<void> {
        await fetch(`${API_BASE}/layout/${layoutId}/deploy`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
        });
    },

    async getPortfolio(layoutId: string): Promise<Portfolio> {
        const res = await fetch(`${API_BASE}/layout/${layoutId}/wallet`);
        if (!res.ok) throw new Error("Failed to fetch wallet");
        const data = await res.json();

        // Backend returns: { status: "OK", wallet: { [multiplexerId: string]: BackendPortfolio } }
        // We need to adapt this to Frontend 'Portfolio' interface: { wallets: Wallet[] }

        if (!data.wallet) {
            return { wallets: [] };
        }

        const backendMap = data.wallet?.wallet;
        if (!backendMap) return { wallets: [] };

        const wallets: any[] = Object.entries(backendMap).map(([id, p]: [string, any]) => {
            const positions: any[] = [];

            if (p.positions && p.positions.holdings) {
                // Note: Rust HashMap<Instrument, f64> serialization key format is tricky.
                // Assuming it's serialized as string. We attempt to parse it or use as-is.
                for (const [key, size] of Object.entries(p.positions.holdings)) {
                    let instrument;
                    try {
                        // Rust serde_json key is usually a string. 
                        // For complex enums, it might be weird. 
                        // Assuming JSON stringified key if custom serializer used, or failure.
                        // But since we are debugging, let's try strict parse.
                        instrument = JSON.parse(key);
                    } catch (e) {
                        // If not JSON, maybe simple string or failed serialization format
                        // Assume Stock for simple strings?
                        instrument = { type: 'Unknown', symbolId: key };
                    }
                    positions.push({ instrument, size });
                }
            }

            const cash = p.cash?.accounts?.["USD"]?.amount || 0;
            return { id, positions, cash };
        });

        return { wallets };
    },
};
