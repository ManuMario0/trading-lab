/**
 * Sidebar.tsx
 * 
 * The palette of available microservices (Nodes) that can be dragged onto the board.
 * Each item in the sidebar represents a "Draft" of a specific service type.
 * 
 * Responsibilities:
 * - Rendering drag-sources for each node type
 * - Setting dataTransfer properties for Drag & Drop logic
 */
import { useEffect, useState } from "react";
import { apiClient } from "../../../../api/client";

export type SymbolId = string;

export type OptionType =
    | { type: 'Call' }
    | { type: 'Put' };

export interface OptionContract {
    underlying_symbol: SymbolId,
    strike_price: number,
    option_type: OptionType,
    expiration_date: string, // YYYY-MM-DD format
    contract_size: number,      // e.g. 100 for standard equity options
}

export interface CurrencyPair {
    base: string,
    quote: string,
}

export type Instrument =
    | { type: 'Stock', symbolId: SymbolId }
    | { type: 'Future', symbolId: SymbolId }
    | { type: 'Option', optionContract: OptionContract }
    | { type: 'Forex', currencyPair: CurrencyPair };


export interface Position {
    instrument: Instrument,
    size: number,
}

export interface Wallet {
    id: string;
    positions: Position[],
    cash: number,
}

export interface Portfolio {
    wallets: Wallet[],
}

const renderPosition = ((position: Position) => {
    switch (position.instrument.type) {
        case 'Stock':
            return <div>{position.instrument.symbolId}</div>;
        case 'Future':
            return <div>{position.instrument.symbolId}</div>;
        case 'Option':
            return <div>{position.instrument.optionContract.underlying_symbol}</div>;
        case 'Forex':
            return <div>{position.instrument.currencyPair.base}/{position.instrument.currencyPair.quote}</div>;
    }
});

export default () => {
    const [portfolio, setPortfolio] = useState<Portfolio>();
    useEffect(() => {
        // 1. Define the fetch function
        const fetchPositions = async () => {
            try {
                // Call your API here
                const data = await apiClient.getPortfolio("default");
                setPortfolio(data);
            } catch (err) {
                console.error("Failed to poll positions", err);
            }
        };
        // 2. Call it immediately so we don't wait for the first interval
        fetchPositions();
        // 3. Set up the interval (e.g., every 1000ms = 1 second)
        const intervalId = setInterval(fetchPositions, 1000);
        // 4. Cleanup: This is crucial! It stops the loop when the component unmounts
        return () => clearInterval(intervalId);
    }, []);

    return (
        <aside style={{
            width: '200px',
            padding: '10px 0', // Remove horizontal padding for cleaner look
            borderLeft: '1px solid var(--color-neon-blue)',
            background: '#13131f',
            overflowY: 'auto',
            maxHeight: '100%',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            gap: '5px'
        }}>
            <div style={{
                color: '#666',
                fontSize: '0.7em',
                textTransform: 'uppercase',
                letterSpacing: '1px',
                marginBottom: '10px',
                width: '100%',
                textAlign: 'center',
                borderBottom: '1px solid #333',
                paddingBottom: '10px'
            }}>
                Positions
            </div>

            <div>
                {portfolio?.wallets?.map((wallet) => (
                    <div key={wallet.id}>
                        <div><strong>{wallet.id}</strong></div>
                        <div>Cash: {wallet.cash}</div>
                        {wallet.positions.map((position, pIndex) => (
                            <div key={pIndex}>
                                [{position.instrument.type}] {renderPosition(position)} : {position.size}
                            </div>
                        ))}
                    </div>
                ))}
            </div>

        </aside>
    )
}
