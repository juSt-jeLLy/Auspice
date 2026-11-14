"use client";

import { Action, FunctionCall } from "@near-js/transactions";
import { setupWalletSelector, type WalletSelector } from "@near-wallet-selector/core";
import { setupHotWallet } from "@near-wallet-selector/hot-wallet";
import { setupIntearWallet } from "@near-wallet-selector/intear-wallet";
import { setupMeteorWallet } from "@near-wallet-selector/meteor-wallet";
import { setupSender } from "@near-wallet-selector/sender";
import { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";
import { factoryId, networkId, rpcUrl } from "@/lib/near";

type WalletState = {
  accountId: string | null;
  accounts: string[];
  ready: boolean;
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  switchAccount: (accountId: string) => void;
  // A raw JSON payload is needed only for exact JSON u64 values (nanosecond
  // timestamps). JavaScript numbers cannot represent those values safely.
  call: (receiverId: string, methodName: string, args: object | string, deposit?: string, gas?: bigint) => Promise<void>;
};

const WalletContext = createContext<WalletState | null>(null);
let selectorPromise: Promise<WalletSelector> | undefined;

function selector() {
  selectorPromise ??= setupWalletSelector({
    network: { networkId, nodeUrl: rpcUrl, helperUrl: "https://helper.testnet.near.org", explorerUrl: "https://testnet.nearblocks.io", indexerUrl: "https://api.kitwallet.app" },
    modules: [setupMeteorWallet(), setupIntearWallet(), setupSender(), setupHotWallet()],
  });
  return selectorPromise;
}

export function WalletProvider({ children }: { children: React.ReactNode }) {
  const [accountId, setAccountId] = useState<string | null>(null);
  const [accounts, setAccounts] = useState<string[]>([]);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    let mounted = true;
    selector().then((walletSelector) => {
      const sync = () => {
        const available = walletSelector.store.getState().accounts;
        const next = available.find((account) => account.active)?.accountId ?? null;
        if (mounted) { setAccountId(next); setAccounts(available.map((account) => account.accountId)); }
      };
      sync();
      setReady(true);
      return walletSelector.store.observable.subscribe(sync);
    }).catch(() => mounted && setReady(true));
    return () => { mounted = false; };
  }, []);

  const connect = useCallback(async () => {
    const walletSelector = await selector();
    const walletModule = walletSelector.store.getState().modules.find((item) => item.metadata.available);
    if (!walletModule) throw new Error("No compatible NEAR wallet was found.");
    const wallet = await walletSelector.wallet(walletModule.id);
    // Selector modules expose a union of wallet-specific sign-in parameter types;
    // every enabled browser/injected module accepts the common contractId shape.
    await wallet.signIn({ contractId: factoryId } as never);
  }, []);
  const disconnect = useCallback(async () => { const wallet = await (await selector()).wallet(); await wallet.signOut(); setAccountId(null); setAccounts([]); }, []);
  const switchAccount = useCallback((next: string) => { selector().then((value) => value.setActiveAccount(next)); }, []);
  const call = useCallback(async (receiverId: string, methodName: string, args: object | string, deposit = "0", gas = 100_000_000_000_000n) => {
    const walletSelector = await selector();
    const wallet = await walletSelector.wallet();
    const encodedArgs = typeof args === "string" ? args : JSON.stringify(args);
    await wallet.signAndSendTransaction({ receiverId, signerId: accountId ?? undefined, actions: [new Action({ functionCall: new FunctionCall({ methodName, args: new TextEncoder().encode(encodedArgs), gas, deposit: BigInt(deposit) }) })] });
  }, [accountId]);
  const value = useMemo(() => ({ accountId, accounts, ready, connect, disconnect, switchAccount, call }), [accountId, accounts, ready, connect, disconnect, switchAccount, call]);
  return <WalletContext.Provider value={value}>{children}</WalletContext.Provider>;
}

export function useWallet() {
  const state = useContext(WalletContext);
  if (!state) throw new Error("useWallet must be used inside WalletProvider");
  return state;
}
