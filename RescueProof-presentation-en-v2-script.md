# RescueProof presentation narration v2

Neural English narration; Pexels photography and actual UI captures. All food data is synthetic.

## 01 / THE HUMAN PROBLEM

Every day, food businesses have safe surplus while nearby community groups are looking for food. The hard part is not finding a good intention. It is coordinating the handoff, keeping the details clear, and showing what actually happened. RescueProof is built for that missing layer of trust.

## 02 / THE OPPORTUNITY

The product goal is simple: make the path from surplus to receipt easy to follow. Donors publish the food and the pickup window. Approved community organizations claim it. Both sides record the handoff and the actual received amount. The result is a usable operational history instead of a trail of disconnected messages.

## 03 / THE WORKFLOW

This is the complete workflow. A donor publishes a listing, an approved organization claims it, both parties confirm the handoff, and a reviewer can inspect the resulting record. Routine users see a normal web application. The integrity check stays in the background until it is useful.

## 04 / WORKING PROTOTYPE

Here is the working prototype. A donor records the quantity, expiry, pickup window, storage requirements, and allergen information before publishing. These details are the operational foundation of a safe handoff. For this presentation, the records are synthetic and no real food or personal contact is involved.

## 05 / HANDOFF

After the claim, the recipient receives the pickup details and generates a time-limited code. The donor validates that code at handoff. The recipient then reports what was actually received. This keeps the donor estimate separate from the recipient confirmation and makes each step visible in the timeline.

## 06 / ARCHITECTURE

The client uses Vue 3 and TypeScript. The API is Rust and Axum. PostgreSQL remains the source of truth for users, organizations, batches, claims, handovers, and proof records. Redis supports rate limiting and short-lived workflow jobs. A worker groups completed events into a Merkle root, while the detailed record stays off chain.

## 07 / WHY WEB3

Web3 is used for one narrow purpose: record integrity. Completed event payloads are hashed into a Merkle tree and a compact root is anchored on Ethereum Sepolia through a relayer. The chain does not receive private addresses or contact details. This design reduces writes, protects privacy, and avoids forcing every participant to manage a wallet.

## 08 / SOCIAL IMPACT

The impact is practical. RescueProof helps small food businesses and community organizations coordinate with less ambiguity. It does not claim to certify food safety or prove a physical delivery. It records the actions, preserves the evidence boundary, and gives local partners a clearer foundation for a pilot.

## 09 / CLOSING

RescueProof turns a fragmented food rescue handoff into a workflow that people can use and reviewers can check. This prototype is open source, testnet-only, and based on synthetic data. The next step is a pilot with accountable food businesses and community organizations. Thank you for reviewing RescueProof.