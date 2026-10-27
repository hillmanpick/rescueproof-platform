# RescueProof Round 2 presentation

Synthetic demonstration data. Stock photographs illustrate context, not a RescueProof pilot. Neural English narration: Ava.

01 / THE HUMAN PROBLEM

Every day, food businesses have safe surplus while nearby community groups are looking for food. The hard part is not finding a good intention. It is coordinating the handoff, keeping the details clear, and showing what actually happened. A message can say that food was available. A spreadsheet can say that someone planned to collect it. Neither one gives everyone the same, reliable timeline. RescueProof is built for that missing layer of trust.

02 / THE PEOPLE

RescueProof is designed around three people in the same handoff. A food business needs to describe what is available and when it must be collected. An approved community organization needs to claim the offer and receive the pickup details at the right moment. An operator needs to see whether the process was completed, disputed, or corrected. Each person needs the same core facts, without exposing more private information than the workflow requires.

03 / THE OPPORTUNITY

The product goal is practical. Make the path from surplus to receipt easy to follow. The donor publishes the food, quantity, pickup window, storage requirements, and allergen notes. An approved organization claims the listing. The donor validates the handoff. The recipient confirms what was actually received. This does not replace an in-person food safety check. It creates a clear operational record around that check, so a coordinator can understand the full story later.

04 / THE WORKFLOW

This is the complete workflow. First, a donor publishes a listing. Second, an approved organization claims it. Third, both sides confirm the handoff and the actual received amount. Finally, a reviewer can inspect the record and its integrity evidence. Routine users see a normal web application. They do not need to understand wallets, gas, or Merkle trees to share food. The technical layer appears when a reviewer needs independent evidence.

05 / DONOR FLOW

Here is the donor flow in the working prototype. The donor selects an organization, names the food, records the quantity and estimated weight, and sets the pickup window. Storage requirements and allergen information are visible parts of the form, not hidden notes in a chat message. The donor can review the details before publishing. This demonstration uses synthetic test data. It does not represent a real food offer or a real contact address.

06 / RECIPIENT FLOW

The recipient sees available food through a separate organization account. Claiming the listing creates an explicit event and prevents two groups from treating the same offer as theirs. Only after the claim succeeds do the private pickup details become available. This is a deliberate product choice. Participants can use the workflow without first creating a crypto wallet, buying tokens, or paying a transaction fee.

07 / HANDOFF CONTROL

At handoff, the recipient generates a time-limited pickup code. The donor checks that code before confirming the collection. Once it is used, the code is consumed and cannot authorize another pickup. This reduces a common coordination mistake: the wrong person arriving for the right listing, or the same listing being treated as open twice. The code is only a workflow control. It is not proof of physical identity, food safety, or successful delivery by itself.

08 / RECEIPT

The donor confirms that the handoff happened. The recipient then enters the actual received weight and confirms receipt. In this synthetic test, the recipient reports two point eight kilograms. RescueProof keeps that reported amount separate from the donor's original estimate. That distinction matters. It prevents an estimate from being presented as completed impact, and it gives operators a way to see where a correction or a dispute came from.

09 / ARCHITECTURE

The client is Vue 3 and TypeScript. The API is Rust and Axum. PostgreSQL is the source of truth for users, organizations, batches, claims, handovers, disputes, and proof records. Redis supports rate limiting and short-lived workflow jobs. Background workers process notifications and evidence batching. This separation keeps ordinary operations responsive and gives the backend one place to enforce authorization, validation, and transaction boundaries.

10 / WHY WEB3

Web3 is used for one narrow purpose: record integrity. Completed event payloads are hashed into a Merkle tree, and a compact root is anchored on Ethereum Sepolia through a relayer. The full record, pickup address, and personal details stay off chain. A reviewer can compare the stored event, its Merkle path, and the public contract root. Putting every field on chain would be more expensive, less private, and harder to correct operationally.

11 / VERIFICATION

This screen shows a separate demonstration batch that was anchored earlier. Each event has an on-chain verified status, a transaction link, and a downloadable verification proof. The application can recompute the event hash and compare the Merkle path with the public root. That gives an external reviewer a way to check whether the stored event changed after anchoring. The proof checks record integrity. It does not certify food quality or physical delivery.

12 / SOCIAL IMPACT

The impact is practical. RescueProof helps small food businesses and community organizations coordinate with less ambiguity. A donor can see what was offered. A recipient can see what was claimed. An operator can see whether the handoff was confirmed or disputed. That shared history can reduce wasted surplus and make a local pilot easier to evaluate. The platform records coordination evidence while leaving food safety and in-person judgment with the participating organizations.

13 / CLOSING

RescueProof turns a fragmented food rescue handoff into a workflow that people can use and reviewers can check. This prototype is open source, testnet-only, and based on synthetic data. The next step is a pilot with accountable food businesses and community organizations. We are using Web3 where it adds public accountability, while keeping routine actions familiar and private. Thank you for reviewing RescueProof.