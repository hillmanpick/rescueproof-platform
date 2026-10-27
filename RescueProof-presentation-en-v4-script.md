# RescueProof Round 2 presentation

Synthetic demonstration data. Stock photographs illustrate context, not a RescueProof pilot. Neural English narration: Andrew. Subtitles: English and Simplified Chinese.

01 / THE HUMAN PROBLEM

Every day, food businesses have safe surplus

每一天，食品商家都会有可安全食用的剩余食物

while nearby community groups are looking for food.

而附近的社区组织仍在寻找食物来源。

The hard part is not finding a good intention.

难点不在于缺少善意，

It is coordinating the handoff, keeping the details clear,

而在于协调交接、厘清细节，

and showing what actually happened.

并留下实际发生过什么的记录。

A message can say that food was available.

一条消息可以说明曾有食物可供领取，

A spreadsheet can say that someone planned to collect it.

一张表格可以说明有人计划前来领取。

Neither one gives everyone the same, reliable timeline.

但它们都难以让各方看到同一条可靠的时间线。

RescueProof is built for that missing layer of trust.

RescueProof 要补上的，正是这一层信任。

02 / THE PEOPLE

RescueProof is designed around three people in the same handoff.

RescueProof 围绕一次交接中的三类参与者设计。

A food business needs to describe what is available

食品商家需要说明有哪些食物，

and when it must be collected.

以及最晚何时需要领取。

An approved community organization needs to claim the offer

经过审核的社区组织需要认领食物，

and receive the pickup details at the right moment.

并在适当的环节获得领取详情。

An operator needs to see whether the process was completed, disputed, or corrected.

运营人员需要了解流程是否完成、存在争议或经过更正。

Each person needs the same core facts,

各方需要共享同一组核心事实，

without exposing more private information than the workflow requires.

同时只披露流程所必需的隐私信息。

03 / THE OPPORTUNITY

The product goal is practical.

产品的目标很务实：

Make the path from surplus to receipt easy to follow.

让剩余食物从发布到签收的每一步都清晰可查。

The donor publishes the food, quantity, pickup window,

捐赠方发布食物信息、数量、领取时段，

storage requirements, and allergen notes.

以及储存要求和过敏原说明。

An approved organization claims the listing.

经过审核的组织认领该条目。

The donor validates the handoff.

捐赠方核验交接，

The recipient confirms what was actually received.

接收方确认实际收到的食物。

This does not replace an in-person food safety check.

这不能替代现场的食品安全检查。

It creates a clear operational record around that check,

它为相关操作建立清晰的记录，

so a coordinator can understand the full story later.

让协调人员事后能够了解完整经过。

04 / THE WORKFLOW

This is the complete workflow.

这是完整的业务流程。

First, a donor publishes a listing.

第一步，捐赠方发布食物条目。

Second, an approved organization claims it.

第二步，经过审核的组织进行认领。

Third, both sides confirm the handoff and the actual received amount.

第三步，双方确认交接和实际接收量。

Finally, a reviewer can inspect the record and its integrity evidence.

最后，审核者可以查看记录及其完整性凭证。

Routine users see a normal web application.

普通用户使用的是熟悉的网页应用。

They do not need to understand wallets, gas, or Merkle trees to share food.

分享食物无需先了解钱包、Gas 费或默克尔树。

The technical layer appears when a reviewer needs independent evidence.

当审核者需要独立核验依据时，技术层才发挥作用。

05 / DONOR FLOW

Here is the donor flow in the working prototype.

这里展示可运行原型中的捐赠方流程。

The donor selects an organization, names the food,

捐赠方选择组织、填写食物名称，

records the quantity and estimated weight, and sets the pickup window.

记录数量和预估重量，并设定领取时段。

Storage requirements and allergen information are visible parts of the form,

储存要求和过敏原信息直接呈现在表单中，

not hidden notes in a chat message.

无需再到聊天消息里翻找。

The donor can review the details before publishing.

捐赠方可以在发布前核对所有详情。

This demonstration uses synthetic test data.

本次演示使用虚构测试数据，

It does not represent a real food offer or a real contact address.

不代表真实的食物供应或联系地址。

06 / RECIPIENT FLOW

The recipient sees available food through a separate organization account.

接收方通过独立的组织账号查看可领取的食物。

Claiming the listing creates an explicit event

认领条目会生成明确的事件记录，

and prevents two groups from treating the same offer as theirs.

防止两个组织同时认为自己已认领同一份食物。

Only after the claim succeeds do the private pickup details become available.

只有认领成功后，才会显示私密的领取详情。

This is a deliberate product choice.

这是产品有意作出的设计选择。

Participants can use the workflow without first creating a crypto wallet,

参与者使用流程时，无需先创建加密钱包，

buying tokens, or paying a transaction fee.

也无需购买代币或支付交易费用。

07 / HANDOFF CONTROL

At handoff, the recipient generates a time-limited pickup code.

交接时，接收方生成有时效的取货码。

The donor checks that code before confirming the collection.

捐赠方核验取货码后，再确认领取。

Once it is used, the code is consumed and cannot authorize another pickup.

取货码一经使用就会失效，不能再次用于领取。

This reduces a common coordination mistake:

这有助于减少常见的协调失误：

the wrong person arriving for the right listing,

例如领取人与条目不匹配，

or the same listing being treated as open twice.

或同一条目被重复当作待领取状态。

The code is only a workflow control.

取货码只是流程控制措施，

It is not proof of physical identity, food safety, or successful delivery by itself.

单靠它无法证明真实身份、食品安全或配送成功。

08 / RECEIPT

The donor confirms that the handoff happened.

捐赠方先确认交接已经发生。

The recipient then enters the actual received weight and confirms receipt.

接收方随后填写实际接收重量并确认签收。

In this synthetic test, the recipient reports two point eight kilograms.

在这组虚构测试中，接收方填报了 2.8 公斤。

RescueProof keeps that reported amount separate from the donor's original estimate.

RescueProof 将该实收量与捐赠方最初的预估分开记录。

That distinction matters.

这个区分很关键。

It prevents an estimate from being presented as completed impact,

它避免把预估量直接当作已经实现的社会成效，

and it gives operators a way to see where a correction or a dispute came from.

也让运营人员能追溯更正或争议的来源。

09 / ARCHITECTURE

The client is Vue 3 and TypeScript.

前端采用 Vue 3 和 TypeScript。

The API is Rust and Axum.

API 服务采用 Rust 和 Axum。

PostgreSQL is the source of truth for users, organizations, batches,

PostgreSQL 统一保存用户、组织、批次，

claims, handovers, disputes, and proof records.

以及认领、交接、争议和凭证记录。

Redis supports rate limiting and short-lived workflow jobs.

Redis 支持请求限流与短时流程任务。

Background workers process notifications and evidence batching.

后台工作进程负责通知和凭证批量处理。

This separation keeps ordinary operations responsive

这样的职责划分让日常操作保持流畅，

and gives the backend one place to enforce authorization,

并让后端集中执行权限控制、

validation, and transaction boundaries.

数据校验与事务边界管理。

10 / WHY WEB3

Web3 is used for one narrow purpose: record integrity.

Web3 专注于一个明确目标：记录完整性。

Completed event payloads are hashed into a Merkle tree,

已完成事件的数据经哈希计算后构建默克尔树，

and a compact root is anchored on Ethereum Sepolia through a relayer.

再由中继服务将精简的树根写入以太坊 Sepolia 测试网。

The full record, pickup address, and personal details stay off chain.

完整记录、领取地址和个人信息保留在链下。

A reviewer can compare the stored event, its Merkle path,

审核者可以比对保存的事件、对应的默克尔路径，

and the public contract root.

以及公开合约中的树根。

Putting every field on chain would be more expensive,

如果把每个字段都写入链上，成本会更高，

less private, and harder to correct operationally.

隐私更难保护，业务更正也会更困难。

11 / VERIFICATION

This screen shows a separate demonstration batch that was anchored earlier.

这里展示的是另一个此前已完成上链的演示批次。

Each event has an on-chain verified status,

每个事件都有链上核验状态，

a transaction link, and a downloadable verification proof.

并提供交易链接和可下载的核验凭证。

The application can recompute the event hash

应用可以重新计算事件哈希，

and compare the Merkle path with the public root.

并将默克尔路径与公开树根进行比对。

That gives an external reviewer a way to check

这让外部审核者能够独立检查：

whether the stored event changed after anchoring.

保存的事件在上链存证后是否发生过变化。

The proof checks record integrity.

该凭证核验的是记录完整性，

It does not certify food quality or physical delivery.

并不认证食品质量或实际配送。

12 / SOCIAL IMPACT

The impact is practical.

我们希望带来的社会影响很务实。

RescueProof helps small food businesses and community organizations

RescueProof 帮助小型食品商家与社区组织，

coordinate with less ambiguity.

减少协调过程中的信息模糊。

A donor can see what was offered.

捐赠方能看到自己提供了什么，

A recipient can see what was claimed.

接收方能看到自己认领了什么，

An operator can see whether the handoff was confirmed or disputed.

运营人员能看到交接是否已确认、是否存在争议。

That shared history can reduce wasted surplus

这份共享历史有助于减少剩余食物浪费，

and make a local pilot easier to evaluate.

也让本地试点更容易评估。

The platform records coordination evidence

平台负责记录协调过程的依据，

while leaving food safety and in-person judgment with the participating organizations.

食品安全和现场判断仍由参与组织负责。

13 / CLOSING

RescueProof turns a fragmented food rescue handoff

RescueProof 将原本零散的食物救助交接，

into a workflow that people can use and reviewers can check.

变成参与者能使用、审核者能核查的工作流程。

This prototype is open source, testnet-only, and based on synthetic data.

该原型开源、仅使用测试网，并基于虚构数据演示。

The next step is a pilot with accountable food businesses and community organizations.

下一步是与负责任的食品商家及社区组织开展试点。

We are using Web3 where it adds public accountability,

我们在能提升公共可核查性的环节使用 Web3，

while keeping routine actions familiar and private.

同时让日常操作保持熟悉，并保护隐私。

Thank you for reviewing RescueProof.

感谢您评审 RescueProof。