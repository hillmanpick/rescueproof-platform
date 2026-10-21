import { reactive } from 'vue'
export interface Organization {id:string;name:string;kind:'donor'|'recipient';status:string}
export interface User {id:string;email:string;name:string;is_admin:boolean}
export interface Batch {id:string;title:string;category:string;quantity:number;unit:string;weight_kg:number;area:string;pickup_start:string;pickup_end:string;expires_at:string;status:string;donor_name:string;claim_id?:string;claim_status?:string;recipient_name?:string;is_donor?:boolean;anchor_status?:string}
export const session=reactive<{user:User|null;organizations:Organization[];loaded:boolean}>({user:null,organizations:[],loaded:false})
export class ApiError extends Error {constructor(public code:string,message:string,public status:number){super(message)}}
const errors:Record<string,string>={UNAUTHENTICATED:'请先登录，或检查邮箱和密码。',FORBIDDEN:'当前组织尚未通过审核，或没有执行该操作的权限。',RATE_LIMITED:'操作太频繁，请一分钟后再试。',BATCH_UNAVAILABLE:'这批食品已被认领或已超过领取时段。',INVALID_STATE:'状态已经变化，请刷新页面后重试。',EMAIL_EXISTS:'这个邮箱已经注册，请直接登录。',INVALID_HANDOVER:'领取码已过期、已使用或与该认领不匹配，请接收方重新生成。',IDEMPOTENCY_MISMATCH:'重复请求的内容有变化，请刷新后重试。'}
export async function api<T=Record<string,unknown>>(path:string,options:RequestInit={}):Promise<T>{
  let response:Response
  try {response=await fetch(`/api/v1${path}`,{...options,credentials:'same-origin',headers:{'Content-Type':'application/json','X-Requested-With':'RescueProof',...options.headers}})} catch {throw new Error('暂时连接不到服务，请检查网络或稍后重试。')}
  const body=await response.json().catch(()=>null)
  if(!response.ok){const code=body?.error?.code??'REQUEST_FAILED';throw new ApiError(code,errors[code]??body?.error?.message??`请求失败 (${response.status})`,response.status)}
  return body as T
}
export async function refreshSession(){try {const d=await api<{user:User;organizations:Organization[]}>('/auth/me');session.user=d.user;session.organizations=d.organizations} catch(e){if(e instanceof ApiError && e.status===401){session.user=null;session.organizations=[]}else{throw e}}finally{session.loaded=true}}
export const post=<T=Record<string,unknown>>(path:string,body:unknown={},key=crypto.randomUUID())=>api<T>(path,{method:'POST',headers:{'Idempotency-Key':key},body:JSON.stringify(body)})
export const labels:Record<string,string>={open:'可认领',claimed:'待领取',handed_over:'待确认收到',completed:'已完成',expired:'已过期',cancelled:'已取消',disputed:'争议处理中',pending:'待审核',approved:'已审核',suspended:'已暂停',bakery:'烘焙食品',produce:'新鲜果蔬',packaged:'包装食品',prepared:'即食餐食',portions:'份',items:'件',boxes:'箱',donor:'食品捐赠方',recipient:'社区接收方',submitted:'交易已提交',confirmed:'已上链核验'}
export const date=(s:string)=>new Intl.DateTimeFormat('zh-CN',{month:'short',day:'numeric',hour:'2-digit',minute:'2-digit'}).format(new Date(s))
export function pickupCode(raw:string):string{try {const data=JSON.parse(raw);if(typeof data.token==='string')return data.token.trim()}catch{/* Manual 64-character tokens are also accepted. */}return raw.trim()}
export function localDateTime(iso:string):string{const d=new Date(iso);const pad=(n:number)=>String(n).padStart(2,'0');return `${d.getFullYear()}-${pad(d.getMonth()+1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`}
