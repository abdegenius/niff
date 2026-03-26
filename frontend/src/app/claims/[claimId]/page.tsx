import { ClaimVotePanel } from '@/components/claims/claim-vote-panel'

interface ClaimPageProps {
  params: { claimId: string }
}

/**
 * /claims/[claimId]
 *
 * Wallet address and currentLedger are passed as props here.
 * In production, replace the stubs below with your wallet context
 * (e.g. Freighter / WalletConnect) and a Horizon ledger sequence fetch.
 */
export default function ClaimPage({ params }: ClaimPageProps) {
  // TODO: replace with wallet context hook (e.g. useFreighter / useWalletConnect)
  const walletAddress: string | null = null
  // TODO: replace with real ledger sequence from Horizon /fee_stats or polling
  const currentLedger = 0

  return (
    <main className="mx-auto max-w-2xl px-4 py-10">
      <h1 className="mb-6 text-xl font-bold">
        Claim vote — <span className="font-mono text-base">{params.claimId}</span>
      </h1>
      <ClaimVotePanel
        claimId={params.claimId}
        walletAddress={walletAddress}
        currentLedger={currentLedger}
      />
    </main>
  )
}
