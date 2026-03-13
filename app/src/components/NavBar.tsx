"use client";

/**
 * Top navigation bar.
 * TODO issue #45: integrate Stellar Wallets Kit for wallet connection button.
 */

import Link from "next/link";
import { useState } from "react";

export function NavBar() {
  const [connected, setConnected] = useState(false);

  return (
    <nav className="fixed top-0 left-0 right-0 bg-white border-b border-gray-200 z-50 h-16">
      <div className="max-w-4xl mx-auto px-4 h-full flex items-center justify-between">
        <div className="flex items-center gap-8">
          <Link href="/" className="text-lg font-bold text-gray-900">
            NebGov
          </Link>
          <div className="flex items-center gap-6 text-sm text-gray-500">
            <Link href="/" className="hover:text-gray-900 transition-colors">
              Proposals
            </Link>
            <Link
              href="/treasury"
              className="hover:text-gray-900 transition-colors"
            >
              Treasury
            </Link>
          </div>
        </div>

        {/* TODO issue #45: replace with StellarWalletsKit connect modal */}
        <button
          onClick={() => setConnected(!connected)}
          className={`text-sm px-4 py-2 rounded-lg font-medium transition-colors ${
            connected
              ? "bg-green-50 text-green-700 border border-green-200"
              : "bg-indigo-600 text-white hover:bg-indigo-700"
          }`}
        >
          {connected ? "G1AB...CD89" : "Connect Wallet"}
        </button>
      </div>
    </nav>
  );
}
