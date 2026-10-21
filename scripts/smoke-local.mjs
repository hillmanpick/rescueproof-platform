import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {randomUUID} from 'node:crypto';
import {createRequire} from 'node:module';
const require=createRequire(new URL('../relayer/package.json',import.meta.url));
const {JsonRpcProvider,Contract,keccak256,toUtf8Bytes,concat}=require('ethers');

const base='http://127.0.0.1:3000/api/v1';
const out=process.env.RESCUEPROOF_TEST_OUTPUT_DIR || 'work/e2e';
mkdirSync(out,{recursive:true});
const run=randomUUID();
const report={run_id:run,started_at:new Date().toISOString(),scope:'Local API with separate demo identities; real Ethereum Sepolia anchoring. Synthetic food only.',checks:[],batches:[],transactions:[]};
const save=()=>writeFileSync(`${out}/RescueProof-闭环实测.json`,JSON.stringify(report,null,2));
const record=(label,detail={})=>{report.checks.push({label,passed:true,...detail});save();console.log(JSON.stringify({check:label,...detail}));};
const creds=JSON.parse(readFileSync('work/demo-accounts.json','utf8')).accounts;
const headers={'content-type':'application/json','origin':'http://localhost:5173','x-requested-with':'RescueProof'};
const sessions={};
async function req(who,path,body,key=randomUUID(),extra={}) {
 const response=await fetch(base+path,{method:body===undefined?'GET':'POST',headers:{...headers,...(sessions[who]?{cookie:sessions[who]}:{}),'idempotency-key':key,...extra},...(body===undefined?{}:{body:JSON.stringify(body)}),signal:AbortSignal.timeout(25000)});
 return {status:response.status,body:await response.json(),cookie:response.headers.get('set-cookie')};
}
async function expect(label,who,path,body,status=200,key=randomUUID(),code) {
 const r=await req(who,path,body,key);
 assert.equal(r.status,status,`${label}: HTTP ${r.status}, error ${r.body.error?.code||'none'}`);
 if(code) assert.equal(r.body.error?.code,code,label);
 record(label,{http_status:r.status,...(code?{error_code:code}:{})});return r.body;
}
const testBatch=(org,title)=>({organization_id:org,title:`【演示】闭环实测 ${run.slice(0,8)} · ${title}`,category:'packaged',quantity:5,unit:'boxes',weight_kg:2.5,area:'上海·徐汇',pickup_address:'闭环演示地址（非真实取货点）',contact:'演示联系人（请勿实际联系）',storage_notes:'仅用于软件测试，没有真实食品',allergens:'演示：请现场核实过敏原',pickup_start:new Date(Date.now()-60000).toISOString(),pickup_end:new Date(Date.now()+3600000).toISOString(),expires_at:new Date(Date.now()+7200000).toISOString(),safety_acknowledged:true});
async function create(org,title){const b=await expect(`发布${title}`,'donor','/batches',testBatch(org,title));report.batches.push({id:b.id,title,expected_status:null});save();return b.id;}
async function detail(who,id){return (await req(who,`/batches/${id}`)).body;}

try {
 for(const role of ['donor','recipient','admin']){
  const a=creds.find(a=>a.email.startsWith(role+'@'));
  const r=await req(null,'/auth/login',{email:a.email,password:a.password});assert.equal(r.status,200,`${role} login failed`);
  sessions[role]=r.cookie.split(';')[0];record(`${role}独立账号登录`,{http_status:r.status});
 }
 const donorMe=(await req('donor','/auth/me')).body,recipientMe=(await req('recipient','/auth/me')).body;
 const donorOrg=donorMe.organizations.find(o=>o.kind==='donor'&&o.status==='approved').id;
 const recipientOrg=recipientMe.organizations.find(o=>o.kind==='recipient'&&o.status==='approved').id;
 const pendingOrg=recipientMe.organizations.find(o=>o.status==='pending')?.id;
 const impactBefore=(await req(null,'/impact')).body;
 const donorBefore=(await req('donor','/dashboard')).body;
 const recipientBefore=(await req('recipient','/dashboard')).body;
 const payload=testBatch(donorOrg,'完整交接');
 await expect('未登录不能发布',null,'/batches',payload,401);
 await expect('接收方不能冒用捐赠机构发布','recipient','/batches',payload,403);
 await expect('缺少食品安全确认被拒绝','donor','/batches',{...payload,safety_acknowledged:false},400,randomUUID(),'SAFETY_REQUIRED');
 const publishKey=randomUUID();
 const published=await expect('发布完整交接批次','donor','/batches',payload,200,publishKey);
 const main=published.id;report.main_batch_id=main;report.batches.push({id:main,title:'完整交接',expected_status:'completed'});save();
 const replay=await expect('重复发布返回原批次','donor','/batches',payload,200,publishKey);
 assert.equal(replay.id,main);
 await expect('同一幂等键不同内容被拒绝','donor','/batches',{...payload,title:payload.title+'修改'},409,publishKey,'IDEMPOTENCY_MISMATCH');
 const listing=(await req(null,'/batches')).body;assert(listing.items.some(b=>b.id===main));record('发布后出现在可认领列表');
 assert.equal((await detail('admin',main)).private,null);record('无关账号不能读取取货地址和联系人');
 await expect('无关账号不能读取核验凭据','admin',`/batches/${main}/proof`,undefined,403);
 if(pendingOrg)await expect('待审核机构不能认领','recipient',`/batches/${main}/claims`,{organization_id:pendingOrg},403);
 await expect('捐赠方不能冒用接收机构认领','donor',`/batches/${main}/claims`,{organization_id:recipientOrg},403);
 const race=await Promise.all([req('recipient',`/batches/${main}/claims`,{organization_id:recipientOrg}),req('recipient',`/batches/${main}/claims`,{organization_id:recipientOrg})]);
 assert.deepEqual(race.map(r=>r.status).sort(),[200,409]);
 const cid=race.find(r=>r.status===200).body.claim.id;record('并发认领只有一个成功',{statuses:race.map(r=>r.status)});
 assert((await detail('recipient',main)).private?.pickup_address);record('认领后接收方获得取货详情');
 await expect('捐赠方不能取消已认领批次','donor',`/batches/${main}/cancel`,{},409);
 await expect('交出前不能确认收货','recipient',`/claims/${cid}/receipt`,{actual_weight_kg:2.4},409,randomUUID(),'INVALID_STATE');
 await expect('捐赠方不能获取接收方领取码','donor',`/claims/${cid}/pickup-token`,{},403);
 const firstToken=await req('recipient',`/claims/${cid}/pickup-token`,{});assert.equal(firstToken.status,200);record('接收方生成领取码');
 const rotated=await req('recipient',`/claims/${cid}/pickup-token`,{});assert.equal(rotated.status,200);assert(firstToken.body.pickup_token!==rotated.body.pickup_token);record('再次生成领取码会轮换');
 await expect('旧领取码无法交接','donor',`/claims/${cid}/handover`,{pickup_token:firstToken.body.pickup_token},409,randomUUID(),'INVALID_HANDOVER');
 await expect('错误领取码无法交接','donor',`/claims/${cid}/handover`,{pickup_token:'f'.repeat(64)},409);
 await expect('接收方不能代替捐赠方确认交出','recipient',`/claims/${cid}/handover`,{pickup_token:rotated.body.pickup_token},403);
 const handoverKey=randomUUID();
 await expect('捐赠方使用有效领取码确认交出','donor',`/claims/${cid}/handover`,{pickup_token:rotated.body.pickup_token},200,handoverKey);
 await expect('重复交出返回原结果','donor',`/claims/${cid}/handover`,{pickup_token:rotated.body.pickup_token},200,handoverKey);
 await expect('已消费领取码不能再次交出','donor',`/claims/${cid}/handover`,{pickup_token:rotated.body.pickup_token},409);
 await expect('交出后不能重新生成领取码','recipient',`/claims/${cid}/pickup-token`,{},409);
 await expect('捐赠方不能冒充接收方签收','donor',`/claims/${cid}/receipt`,{actual_weight_kg:2.4},403);
 await expect('负数签收重量被拒绝','recipient',`/claims/${cid}/receipt`,{actual_weight_kg:-1},400);
 const receiptKey=randomUUID();
 await expect('接收方确认收到2.4kg','recipient',`/claims/${cid}/receipt`,{actual_weight_kg:2.4},200,receiptKey);
 await expect('重复签收返回原结果','recipient',`/claims/${cid}/receipt`,{actual_weight_kg:2.4},200,receiptKey);
 await expect('不同请求不能重复完成签收','recipient',`/claims/${cid}/receipt`,{actual_weight_kg:2.4},409);
 assert.equal((await detail('donor',main)).batch.status,'completed');assert.equal((await detail('recipient',main)).batch.status,'completed');record('双方查看均为已完成');
 const mainEvents=(await req('donor',`/batches/${main}/events`)).body.items;
 assert.deepEqual(mainEvents.map(e=>e.kind),['batch.created','claim.created','handover.donor_confirmed','handover.completed']);record('完整交接仅产生4条事件，无重复记录');
 const after=(await req(null,'/impact')).body;
 assert.equal(after.completed_batches,impactBefore.completed_batches+1);assert.equal(after.completed_claims,impactBefore.completed_claims+1);assert(Math.abs(after.completed_weight_kg-impactBefore.completed_weight_kg-2.4)<1e-6);record('统计只增加一次完成与2.4kg',{before:impactBefore,after});

 const cancelled=await create(donorOrg,'取消与重新认领');
 const c1=(await expect('分支批次首次认领','recipient',`/batches/${cancelled}/claims`,{organization_id:recipientOrg})).claim.id;
 await expect('接收方取消认领','recipient',`/claims/${c1}/cancel`,{});
 assert.equal((await detail('donor',cancelled)).batch.status,'open');record('取消认领后批次重新开放');
 assert.equal((await detail('recipient',cancelled)).private,null);record('取消后撤销接收方私密取货信息访问');
 const c2=(await expect('取消后可再次认领','recipient',`/batches/${cancelled}/claims`,{organization_id:recipientOrg})).claim.id;assert(c1!==c2);
 await expect('取消第二次认领','recipient',`/claims/${c2}/cancel`,{});
 await expect('捐赠方取消开放批次','donor',`/batches/${cancelled}/cancel`,{});
 await expect('已取消批次不可认领','recipient',`/batches/${cancelled}/claims`,{organization_id:recipientOrg},409);
 report.batches.find(b=>b.id===cancelled).expected_status='cancelled';

 const disputed=await create(donorOrg,'争议与管理员处理');
 const c3=(await expect('认领争议测试批次','recipient',`/batches/${disputed}/claims`,{organization_id:recipientOrg})).claim.id;
 await expect('无关账号不能发起争议','admin',`/claims/${c3}/disputes`,{reason:'演示异常，非真实食品问题'},403);
 const dispute=(await expect('接收方发起争议','recipient',`/claims/${c3}/disputes`,{reason:'演示异常，非真实食品问题'})).dispute_id;
 await expect('普通用户不能处理争议','donor',`/admin/disputes/${dispute}/resolve`,{outcome:'cancelled',note:'演示管理员处理记录'},403);
 await expect('管理员不能凭空完成未签收交接','admin',`/admin/disputes/${dispute}/resolve`,{outcome:'completed',note:'演示管理员处理记录'},409,randomUUID(),'NO_RECEIPT');
 const resolutionKey=randomUUID();
 await expect('管理员处理争议并取消交接','admin',`/admin/disputes/${dispute}/resolve`,{outcome:'cancelled',note:'演示管理员处理记录，不对应真实食品'},200,resolutionKey);
 await expect('重复处理争议返回原结果','admin',`/admin/disputes/${dispute}/resolve`,{outcome:'cancelled',note:'演示管理员处理记录，不对应真实食品'},200,resolutionKey);
 report.batches.find(b=>b.id===disputed).expected_status='cancelled';save();

 console.log(JSON.stringify({stage:'waiting_for_worker_and_chain',batch_ids:report.batches.map(b=>b.id)}));
 let finalProofs;let poll=0;
 const deadline=Date.now()+180000;
 while(Date.now()<deadline){
  const all=await Promise.all(report.batches.map(async b=>({batch:b,events:(await req('donor',`/batches/${b.id}/events`)).body.items,proof:(await req('donor',`/batches/${b.id}/proof`)).body.items})));
  if(all.every(x=>x.proof.length===x.events.length&&x.proof.every(p=>p.status==='confirmed'))){finalProofs=all;break;}
  if(poll++%6===0)console.log(JSON.stringify({stage:'confirmation_pending',counts:all.map(x=>({batch_id:x.batch.id,events:x.events.length,proofs:x.proof.length,confirmed:x.proof.filter(p=>p.status==='confirmed').length}))}));
  await new Promise(resolve=>setTimeout(resolve,3000));
 }
 assert(finalProofs,'Timed out waiting for newly generated events to anchor');
 const provider=new JsonRpcProvider('https://sepolia.drpc.org');
 try{
  assert.equal(BigInt(await provider.send('eth_chainId',[])),11155111n);
  const checked=new Set();let count=0;
  for(const {batch,events,proof}of finalProofs){
   assert.equal((await detail('donor',batch.id)).batch.status,batch.expected_status);
   const expected=new Set(events.map(e=>e.id));
   for(const p of proof){
    assert(expected.delete(p.event_id),'Duplicate or unexpected proof');
    const leaf=keccak256(keccak256(toUtf8Bytes(p.canonical_payload)));assert.equal(leaf,p.leaf_hash);
    const fold=s=>p.proof.reduce((a,b)=>keccak256(concat([a,b].sort())),keccak256(keccak256(toUtf8Bytes(s))));
    assert.equal(fold(p.canonical_payload),p.root);assert.notEqual(fold(p.canonical_payload+'tampered'),p.root);
    if(!checked.has(p.transaction_hash)){
     const receipt=await provider.getTransactionReceipt(p.transaction_hash);assert(receipt&&receipt.status===1);assert.equal(receipt.to.toLowerCase(),p.contract_address.toLowerCase());assert.equal(receipt.blockNumber,p.block_number);
     const c=new Contract(p.contract_address,['function roots(bytes32) view returns(uint256)','event RootAnchored(bytes32 indexed root,uint256 timestamp)'],provider);
     assert(await c.roots(p.root,{blockTag:p.block_number})>0n);
     assert(receipt.logs.some(l=>{try{const e=c.interface.parseLog(l);return e?.name==='RootAnchored'&&e.args.root===p.root;}catch{return false;}}));
     report.transactions.push({transaction_hash:p.transaction_hash,root:p.root,block_number:p.block_number,contract_address:p.contract_address,chain_id:p.chain_id,confirmations:await receipt.confirmations(),explorer_url:`https://sepolia.etherscan.io/tx/${p.transaction_hash}`});checked.add(p.transaction_hash);
    }
    count++;
   }
   assert.equal(expected.size,0);writeFileSync(`${out}/closed-loop-${batch.id}-proof.json`,JSON.stringify(proof,null,2));
  }
  record('本轮所有事件均获得真实Sepolia回执和独立RPC根校验',{evidence_count:count,transactions:checked.size});
  record('本轮全部Merkle路径通过，篡改凭据均被拒绝',{valid:count,tampered_rejected:count});
 }finally{provider.destroy();}
 const donorAfter=(await req('donor','/dashboard')).body,recipientAfter=(await req('recipient','/dashboard')).body;
 assert(donorAfter.notifications.some(n=>!donorBefore.notifications.some(old=>old.id===n.id)));assert(recipientAfter.notifications.some(n=>!recipientBefore.notifications.some(old=>old.id===n.id)));record('双方均收到本轮新交接通知');
 for(const role of ['donor','recipient','admin']){await expect(`${role}退出登录`,role,'/auth/logout',{});await expect(`${role}退出后旧会话失效`,role,'/auth/me',undefined,401);}
 report.finished_at=new Date().toISOString();report.passed=true;save();console.log(JSON.stringify({result:'PASS',checks:report.checks.length,main_batch_id:main,transaction_count:report.transactions.length,report:`${out}/RescueProof-闭环实测.json`}));
}catch(error){report.passed=false;report.failure=error.message;report.finished_at=new Date().toISOString();save();console.error(JSON.stringify({result:'FAIL',message:error.message,main_batch_id:report.main_batch_id}));process.exitCode=1;}
