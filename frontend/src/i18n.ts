import { computed, reactive, watchEffect } from "vue";
import { workflow } from "./locales/workflow";

export type Locale = "zh" | "en" | "fr" | "ja";
const supported = ["zh", "en", "fr", "ja"] as const;
export function isLocale(value: unknown): value is Locale {
  return supported.some((code) => code === value);
}
function initialLocale(): Locale {
  try {
    const saved = localStorage.getItem("rescueproof-locale");
    if (isLocale(saved)) return saved;
  } catch {
    /* Storage can be disabled; switching languages still works. */
  }
  const preferred = navigator.languages
    ?.map((language) => language.split("-")[0])
    .find(isLocale);
  return preferred && isLocale(preferred) ? preferred : "zh";
}
export const locale = reactive<{ value: Locale }>({ value: initialLocale() });

const messages: Record<Locale, Record<string, string>> = {
  zh: {
    notice: "让余量成为馈赠，让每一份善意有迹可循。",
    discover: "发现余量",
    how: "如何参与",
    workspace: "我的工作台",
    admin: "组织管理",
    login: "登录",
    share: "分享余量",
    logout: "退出登录",
    heroEyebrow: "小小行动，共同改变。",
    heroTitle: "好食物，",
    heroTitle2: "值得一个",
    heroTitle3: "好去处。",
    heroText:
      "连接一份余量与一份需要。让社区里的好食物被珍惜，让每一次交接清晰可循。",
    findFood: "发现附近余量",
    shareFood: "我有食物想分享",
    trust: "为商家、社区组织和更好的明天而连接",
    impactIntro: "每一份被珍惜的食物，都是社区向前的一小步。",
    completedBatches: "已完成食品交接",
    receivedWeight: "接收方填报的交接重量",
    completedClaims: "双方已确认的认领",
    impactNote: "试点真实记录 · 不将预计量当作实际影响",
    findGood: "发现社区好食物",
    sectionTitle: "余量不多，心意刚好。",
    sectionText: "发现社区里等待被接住的好食物。",
    network: "社区余量分享网络",
    all: "全部余量",
    bakery: "烘焙食品",
    produce: "新鲜果蔬",
    packaged: "包装食品",
    prepared: "即食餐食",
    search: "搜索食品、地区或商家",
    loading: "正在读取…",
    noConnection: "暂时没有连接上",
    retry: "重新加载",
    noMatch: "还没有匹配的余量",
    firstShare: "第一份分享，正在等待你。",
    searchAgain: "换一个关键词或类别再看看。",
    noBatch: "当前暂无可认领批次。加入社区，发布第一份可分享的好食物。",
    publish: "发布食品余量",
    footnote:
      "食品数量与可领取时段由发布方提供。认领前请与发布方确认储存要求及过敏原信息。",
    howEyebrow: "一起，让分享更简单",
    howTitle: "三步，让善意抵达。",
    howText: "不需要钱包，不需要复杂操作。从一个真实的社区连接开始。",
    step1: "分享一份余量",
    step1Text: "商家填写食品信息、数量和领取时段，给仍然美好的食物一个新机会。",
    step2: "与社区相连接",
    step2Text: "经审核的社区组织认领食品，确认领取地点，在约定时间完成交接。",
    step3: "让每份善意可循",
    step3Text: "双方确认真实交接，记录重量与状态，共同积累清晰的社区影响记录。",
    joinEyebrow: "让美好在社区生长",
    joinTitle: "下一份温暖，可以从你开始。",
    joinText: "你的一点余量，也许正是社区需要的一点力量。",
    join: "加入食品分享网络",
    footer: "食物不应被浪费，善意值得被看见。",
    footerMeta: "从一份余量，到一份温暖。",
    footerSafety:
      "食品安全与现场交接由参与组织负责。记录证明用于核验数据完整性。",
    viewDetails: "查看详情",
    beforePickup: "前领取",
    "label.open": "可认领",
    "label.claimed": "待领取",
    "label.handed_over": "待确认收到",
    "label.completed": "已完成",
    "label.expired": "已过期",
    "label.cancelled": "已取消",
    "label.disputed": "争议处理中",
    "label.pending": "待审核",
    "label.approved": "已审核",
    "label.suspended": "已暂停",
    "label.bakery": "烘焙食品",
    "label.produce": "新鲜果蔬",
    "label.packaged": "包装食品",
    "label.prepared": "即食餐食",
    "label.portions": "份",
    "label.items": "件",
    "label.boxes": "箱",
    "label.donor": "食品捐赠方",
    "label.recipient": "社区接收方",
    "label.submitted": "交易已提交",
    "label.confirmed": "已上链核验",
    authWelcome: "欢迎加入社区",
    authTitle: "一份食物，一座社区的善意连接。",
    authCopy: "每一个小小的分享，都可以让好食物走得更远。",
    authSecurity: "组织先审核，交接双确认。",
    authJoin: "加入 RescueProof",
    authHello: "很高兴，再见到你。",
    authJoinCopy: "从你的组织开始，让余量成为馈赠。",
    authLoginCopy: "登录你的社区食品分享工作台。",
    yourName: "你的姓名",
    namePlaceholder: "怎么称呼你？",
    email: "邮箱",
    password: "密码",
    orgName: "组织名称",
    participate: "你希望如何参与？",
    donorRole: "分享食品的商家",
    recipientRole: "领取食品的组织",
    reviewNote:
      "新组织注册后需运营人员审核。邮箱用于登录，本版本尚未提供邮箱验证或自助重置密码。",
    working: "正在处理…",
    createAccount: "创建账户",
    loginWorkspace: "登录工作台",
    alreadyJoined: "已经加入了？",
    firstTime: "第一次来？",
    goLogin: "去登录",
    createOrg: "创建组织账户",
    workspaceGreeting: "今天也有好事发生。",
    workspaceTitle: "你的社区工作台",
    workspaceCopy: "从发布到交接，照看每一份食品的旅程。",
    reading: "正在读取…",
    pendingOrg:
      "组织审核通过后即可发布或认领食品。请联系试点运营人员完成身份确认。",
    foodJourney: "我的食品旅程",
    startKindness: "从第一份善意开始。",
    workspaceEmpty: "这里会显示你发布或认领的食品批次。",
    messages: "社区消息",
    noMessages: "暂时没有新消息。认领和交接发生时，通知会出现在这里。",
    postedByMe: "我发布的",
    claimedByMe: "我认领的",
    recipientOrg: "接收组织：",
    shareButton: "分享食品",
  },
  en: {
    notice: "Let surplus become a gift, with every act of care accounted for.",
    discover: "Find food",
    how: "How it works",
    workspace: "My workspace",
    admin: "Organization admin",
    login: "Log in",
    share: "Share surplus",
    logout: "Log out",
    heroEyebrow: "SMALL ACTIONS. SHARED IMPACT.",
    heroTitle: "Good food",
    heroTitle2: "deserves a",
    heroTitle3: "good home.",
    heroText:
      "Connect surplus with a community that needs it. Keep good food in use and every handoff clear.",
    findFood: "Find food nearby",
    shareFood: "I have food to share",
    trust:
      "Connecting food businesses, community groups, and a better tomorrow",
    impactIntro: "Every rescued meal is a small step forward for a community.",
    completedBatches: "Completed handoffs",
    receivedWeight: "Weight reported by recipients",
    completedClaims: "Claims confirmed by both sides",
    impactNote: "Pilot records · estimates are not counted as actual impact",
    findGood: "FIND GOOD FOOD",
    sectionTitle: "A little surplus can mean a lot.",
    sectionText: "Find good food waiting for a new home nearby.",
    network: "Community food-sharing network",
    all: "All food",
    bakery: "Bakery",
    produce: "Fresh produce",
    packaged: "Packaged food",
    prepared: "Prepared meals",
    search: "Search food, area, or business",
    loading: "Loading…",
    noConnection: "Could not connect",
    retry: "Try again",
    noMatch: "No matching food yet",
    firstShare: "Your first share is waiting to happen.",
    searchAgain: "Try another search or category.",
    noBatch:
      "No claimable batches right now. Join the community and share the first one.",
    publish: "Share surplus food",
    footnote:
      "Food quantity and pickup times are provided by donors. Confirm storage and allergen details before claiming.",
    howEyebrow: "A LITTLE EASIER, TOGETHER",
    howTitle: "Three steps to make care count.",
    howText:
      "No wallet or complicated setup. Just a real connection in your community.",
    step1: "Share surplus food",
    step1Text:
      "Businesses add food details, quantity, and pickup times to give good food another chance.",
    step2: "Connect with community",
    step2Text:
      "Approved community groups claim food, confirm pickup, and meet at the agreed time.",
    step3: "Keep every handoff clear",
    step3Text:
      "Both sides confirm the handoff and record weight and status for a clear community impact trail.",
    joinEyebrow: "GROW SOMETHING GOOD",
    joinTitle: "The next good thing can start with you.",
    joinText: "Your surplus might be exactly what your community needs.",
    join: "Join the food-sharing network",
    footer: "Food deserves to be shared. Care deserves to be seen.",
    footerMeta: "From surplus to something meaningful.",
    footerSafety:
      "Participating organizations handle food safety and in-person handoffs. Records verify data integrity.",
    viewDetails: "View details",
    beforePickup: "Pickup by",
    "label.open": "Available",
    "label.claimed": "Claimed",
    "label.handed_over": "Awaiting receipt",
    "label.completed": "Completed",
    "label.expired": "Expired",
    "label.cancelled": "Cancelled",
    "label.disputed": "In dispute",
    "label.pending": "Pending review",
    "label.approved": "Approved",
    "label.suspended": "Suspended",
    "label.bakery": "Bakery",
    "label.produce": "Fresh produce",
    "label.packaged": "Packaged food",
    "label.prepared": "Prepared meals",
    "label.portions": "portions",
    "label.items": "items",
    "label.boxes": "boxes",
    "label.donor": "Food donor",
    "label.recipient": "Community recipient",
    "label.submitted": "Transaction submitted",
    "label.confirmed": "On-chain verified",
    authWelcome: "WELCOME TO THE COMMUNITY",
    authTitle: "One meal can connect a community.",
    authCopy: "Every small act of sharing helps good food go further.",
    authSecurity: "Organizations reviewed. Handoffs confirmed by both sides.",
    authJoin: "Join RescueProof",
    authHello: "Good to see you again.",
    authJoinCopy: "Start with your organization and turn surplus into a gift.",
    authLoginCopy: "Log in to your community food-sharing workspace.",
    yourName: "Your name",
    namePlaceholder: "What should we call you?",
    email: "Email",
    password: "Password",
    orgName: "Organization name",
    participate: "How would you like to participate?",
    donorRole: "Business sharing food",
    recipientRole: "Organization receiving food",
    reviewNote:
      "New organizations need operator approval. Email verification and self-service password reset are not available yet.",
    working: "Working…",
    createAccount: "Create account",
    loginWorkspace: "Log in",
    alreadyJoined: "Already a member?",
    firstTime: "New here?",
    goLogin: "Log in",
    createOrg: "Create an organization account",
    workspaceGreeting: "Good things are happening today.",
    workspaceTitle: "Your community workspace",
    workspaceCopy: "Follow each batch from sharing to handoff.",
    reading: "Loading…",
    pendingOrg:
      "Your organization can share or claim food after approval. Contact the pilot operator to verify its identity.",
    foodJourney: "My food journey",
    startKindness: "Start with your first share.",
    workspaceEmpty: "Batches you share or claim will appear here.",
    messages: "Community updates",
    noMessages: "No updates yet. Claims and handoffs will appear here.",
    postedByMe: "Shared by me",
    claimedByMe: "Claimed by me",
    recipientOrg: "Recipient: ",
    shareButton: "Share food",
  },
  fr: {
    notice: "Faisons du surplus un don et traçons chaque geste solidaire.",
    discover: "Découvrir",
    how: "Comment ça marche",
    workspace: "Mon espace",
    admin: "Gestion des organisations",
    login: "Connexion",
    share: "Donner un surplus",
    logout: "Déconnexion",
    heroEyebrow: "PETITS GESTES. IMPACT PARTAGÉ.",
    heroTitle: "Les bons aliments",
    heroTitle2: "méritent un",
    heroTitle3: "bel avenir.",
    heroText:
      "Relions les surplus aux communautés qui en ont besoin. Préservons les aliments et traçons chaque remise.",
    findFood: "Trouver près de moi",
    shareFood: "J’ai des aliments à donner",
    trust: "Relier commerces, associations et avenir meilleur",
    impactIntro: "Chaque repas sauvé fait avancer la communauté.",
    completedBatches: "Remises terminées",
    receivedWeight: "Poids déclaré par les bénéficiaires",
    completedClaims: "Réservations confirmées par les deux parties",
    impactNote: "Données du pilote · les estimations ne sont pas comptées",
    findGood: "DE BONS ALIMENTS À PARTAGER",
    sectionTitle: "Un petit surplus, un grand geste.",
    sectionText: "Découvrez les aliments qui attendent près de chez vous.",
    network: "Réseau solidaire alimentaire",
    all: "Tout",
    bakery: "Boulangerie",
    produce: "Produits frais",
    packaged: "Produits emballés",
    prepared: "Plats préparés",
    search: "Rechercher un aliment, un quartier ou un commerce",
    loading: "Chargement…",
    noConnection: "Connexion impossible",
    retry: "Réessayer",
    noMatch: "Aucun résultat",
    firstShare: "Votre premier don n’attend que vous.",
    searchAgain: "Essayez un autre terme ou une autre catégorie.",
    noBatch:
      "Aucun lot disponible. Rejoignez le réseau et proposez le premier don.",
    publish: "Proposer un surplus",
    footnote:
      "Les quantités et horaires sont indiqués par les donateurs. Vérifiez conservation et allergènes avant de réserver.",
    howEyebrow: "ENSEMBLE, C’EST PLUS SIMPLE",
    howTitle: "Trois étapes pour agir.",
    howText:
      "Pas de portefeuille ni de démarche complexe. Un lien concret dans votre quartier.",
    step1: "Proposer un surplus",
    step1Text:
      "Les commerces indiquent les aliments, les quantités et les horaires pour leur offrir une seconde chance.",
    step2: "Relier la communauté",
    step2Text:
      "Les associations agréées réservent, confirment le lieu et récupèrent les aliments à l’heure prévue.",
    step3: "Tracer chaque remise",
    step3Text:
      "Les deux parties confirment la remise et consignent le poids et le statut.",
    joinEyebrow: "FAISONS GRANDIR LA SOLIDARITÉ",
    joinTitle: "Le prochain geste commence avec vous.",
    joinText: "Votre surplus est peut-être la ressource qu’attend le quartier.",
    join: "Rejoindre le réseau",
    footer: "Les aliments se partagent, la solidarité se voit.",
    footerMeta: "Du surplus à un geste solidaire.",
    footerSafety:
      "Les organisations participantes gèrent la sécurité alimentaire et les remises. Les preuves vérifient l’intégrité des données.",
    viewDetails: "Voir le détail",
    beforePickup: "À récupérer avant",
    "label.open": "Disponible",
    "label.claimed": "Réservé",
    "label.handed_over": "En attente de réception",
    "label.completed": "Terminé",
    "label.expired": "Expiré",
    "label.cancelled": "Annulé",
    "label.disputed": "Litige en cours",
    "label.pending": "En attente",
    "label.approved": "Approuvé",
    "label.suspended": "Suspendu",
    "label.bakery": "Boulangerie",
    "label.produce": "Produits frais",
    "label.packaged": "Produits emballés",
    "label.prepared": "Plats préparés",
    "label.portions": "portions",
    "label.items": "articles",
    "label.boxes": "caisses",
    "label.donor": "Donateur alimentaire",
    "label.recipient": "Association bénéficiaire",
    "label.submitted": "Transaction soumise",
    "label.confirmed": "Vérifié sur la chaîne",
    authWelcome: "BIENVENUE DANS LA COMMUNAUTÉ",
    authTitle: "Un aliment peut rapprocher tout un quartier.",
    authCopy: "Chaque petit don aide les bons aliments à aller plus loin.",
    authSecurity:
      "Organisations vérifiées. Remises confirmées par les deux parties.",
    authJoin: "Rejoindre RescueProof",
    authHello: "Heureux de vous revoir.",
    authJoinCopy: "Commencez avec votre organisation et partagez vos surplus.",
    authLoginCopy: "Connectez-vous à votre espace solidaire.",
    yourName: "Votre nom",
    namePlaceholder: "Comment vous appeler ?",
    email: "E-mail",
    password: "Mot de passe",
    orgName: "Nom de l’organisation",
    participate: "Comment souhaitez-vous participer ?",
    donorRole: "Commerce donateur",
    recipientRole: "Association bénéficiaire",
    reviewNote:
      "Les nouvelles organisations doivent être vérifiées par l’équipe. La vérification d’e-mail et la réinitialisation autonome ne sont pas encore disponibles.",
    working: "Traitement…",
    createAccount: "Créer un compte",
    loginWorkspace: "Connexion",
    alreadyJoined: "Déjà membre ?",
    firstTime: "Première visite ?",
    goLogin: "Se connecter",
    createOrg: "Créer un compte organisation",
    workspaceGreeting: "De belles choses se passent aujourd’hui.",
    workspaceTitle: "Votre espace communautaire",
    workspaceCopy: "Suivez chaque lot, du don à la remise.",
    reading: "Chargement…",
    pendingOrg:
      "Votre organisation pourra donner ou réserver après validation. Contactez l’équipe pilote pour vérifier son identité.",
    foodJourney: "Mon parcours alimentaire",
    startKindness: "Commencez par votre premier don.",
    workspaceEmpty: "Les lots que vous donnez ou réservez apparaîtront ici.",
    messages: "Messages de la communauté",
    noMessages: "Aucun message. Les réservations et remises apparaîtront ici.",
    postedByMe: "Donné par moi",
    claimedByMe: "Réservé par moi",
    recipientOrg: "Bénéficiaire : ",
    shareButton: "Donner des aliments",
  },
  ja: {
    notice: "余剰食品を贈り物に。思いやりの記録を残します。",
    discover: "食品を探す",
    how: "利用方法",
    workspace: "マイページ",
    admin: "団体管理",
    login: "ログイン",
    share: "食品を提供",
    logout: "ログアウト",
    heroEyebrow: "小さな行動で、地域を変える。",
    heroTitle: "おいしい食べ物に",
    heroTitle2: "新しい",
    heroTitle3: "行き先を。",
    heroText:
      "余剰食品を必要とする地域へ。食品を大切に届け、受け渡しを記録します。",
    findFood: "近くの食品を探す",
    shareFood: "食品を提供する",
    trust: "お店と地域団体をつなぎ、よりよい明日へ",
    impactIntro: "救われた一食が、地域の一歩になります。",
    completedBatches: "完了した受け渡し",
    receivedWeight: "受取団体が報告した重量",
    completedClaims: "双方が確認した申込み",
    impactNote: "実証記録 · 予測値は実績に含みません",
    findGood: "地域で食品を見つける",
    sectionTitle: "少しの余剰が、大きな思いやりに。",
    sectionText: "地域で新しい行き先を待つ食品を探しましょう。",
    network: "地域食品シェアネットワーク",
    all: "すべて",
    bakery: "パン・菓子",
    produce: "生鮮食品",
    packaged: "包装食品",
    prepared: "調理済み食品",
    search: "食品・地域・店舗を検索",
    loading: "読み込み中…",
    noConnection: "接続できません",
    retry: "再読み込み",
    noMatch: "該当する食品はありません",
    firstShare: "最初の提供をお待ちしています。",
    searchAgain: "別のキーワードや分類をお試しください。",
    noBatch:
      "現在申込み可能な食品はありません。地域に参加して食品を提供しましょう。",
    publish: "余剰食品を提供",
    footnote:
      "食品の数量と受取時間は提供者が登録しています。申込み前に保存方法とアレルゲンをご確認ください。",
    howEyebrow: "一緒なら、もっと簡単",
    howTitle: "思いやりを届ける3つの手順。",
    howText: "ウォレットも複雑な操作も不要。地域のつながりから始まります。",
    step1: "余剰食品を提供",
    step1Text:
      "店舗が食品情報、数量、受取時間を登録し、おいしい食品に新しい機会を届けます。",
    step2: "地域とつながる",
    step2Text: "承認済みの地域団体が申込み、場所と時間を確認して受け取ります。",
    step3: "受け渡しを記録",
    step3Text:
      "双方で受け渡しを確認し、重量と状態を記録して地域への貢献を残します。",
    joinEyebrow: "地域の良い循環を育てる",
    joinTitle: "次の思いやりは、あなたから。",
    joinText: "あなたの少しの余剰が、地域の力になるかもしれません。",
    join: "食品シェアに参加",
    footer: "食品を無駄にせず、思いやりを見える形に。",
    footerMeta: "余剰から、あたたかいつながりへ。",
    footerSafety:
      "食品安全と現地での受け渡しは参加団体が確認します。記録はデータの完全性を検証します。",
    viewDetails: "詳細を見る",
    beforePickup: "受取期限",
    "label.open": "受付中",
    "label.claimed": "申込み済み",
    "label.handed_over": "受取確認待ち",
    "label.completed": "完了",
    "label.expired": "期限切れ",
    "label.cancelled": "キャンセル",
    "label.disputed": "確認中",
    "label.pending": "審査待ち",
    "label.approved": "承認済み",
    "label.suspended": "停止中",
    "label.bakery": "パン・菓子",
    "label.produce": "生鮮食品",
    "label.packaged": "包装食品",
    "label.prepared": "調理済み食品",
    "label.portions": "食分",
    "label.items": "個",
    "label.boxes": "箱",
    "label.donor": "食品提供者",
    "label.recipient": "地域の受取団体",
    "label.submitted": "取引送信済み",
    "label.confirmed": "チェーン上で検証済み",
    authWelcome: "コミュニティへようこそ",
    authTitle: "一つの食品が、地域をつなぎます。",
    authCopy: "小さな提供が、おいしい食品をもっと遠くへ届けます。",
    authSecurity: "団体を審査し、双方で受け渡しを確認します。",
    authJoin: "RescueProofに参加",
    authHello: "おかえりなさい。",
    authJoinCopy: "団体から始めて、余剰食品を贈り物に。",
    authLoginCopy: "地域食品シェアのマイページにログイン。",
    yourName: "お名前",
    namePlaceholder: "お名前を入力",
    email: "メールアドレス",
    password: "パスワード",
    orgName: "団体名",
    participate: "参加方法を選択",
    donorRole: "食品を提供する店舗",
    recipientRole: "食品を受け取る団体",
    reviewNote:
      "新規団体は運営者の審査が必要です。メール確認とパスワード再設定は現在ご利用いただけません。",
    working: "処理中…",
    createAccount: "アカウント作成",
    loginWorkspace: "ログイン",
    alreadyJoined: "すでに登録済みですか？",
    firstTime: "初めてですか？",
    goLogin: "ログイン",
    createOrg: "団体アカウントを作成",
    workspaceGreeting: "今日も良いことがありますように。",
    workspaceTitle: "地域ワークスペース",
    workspaceCopy: "提供から受け渡しまで、食品の流れを確認できます。",
    reading: "読み込み中…",
    pendingOrg:
      "審査後に食品の提供や申込みができます。本人確認のため運営者にご連絡ください。",
    foodJourney: "食品の履歴",
    startKindness: "最初の提供から始めましょう。",
    workspaceEmpty: "提供または申込みした食品がここに表示されます。",
    messages: "地域のお知らせ",
    noMessages: "お知らせはありません。申込みや受け渡しがここに表示されます。",
    postedByMe: "提供した食品",
    claimedByMe: "申込んだ食品",
    recipientOrg: "受取団体：",
    shareButton: "食品を提供",
  },
};

const workflowMessages: Record<string, readonly string[]> = workflow;
export function t(
  key: string,
  params: Record<string, string | number> = {},
): string {
  const message =
    messages[locale.value][key] ??
    workflowMessages[key]?.[supported.indexOf(locale.value)] ??
    messages.en[key] ??
    key;
  return message.replace(/\{(\w+)\}/g, (token, name: string) =>
    String(params[name] ?? token),
  );
}
export const localizedLabels = new Proxy<Record<string, string>>(
  {},
  {
    get: (_target, key) =>
      typeof key === "string"
        ? (messages[locale.value][`label.${key}`] ?? key)
        : undefined,
  },
);
export function setLocale(value: Locale) {
  if (!isLocale(value)) return;
  locale.value = value;
  try {
    localStorage.setItem("rescueproof-locale", value);
  } catch {
    /* Optional persistence. */
  }
}
export const languageOptions = [
  { value: "zh", label: "中文" },
  { value: "en", label: "English" },
  { value: "fr", label: "Français" },
  { value: "ja", label: "日本語" },
] as const;
export const localeDate = computed(() =>
  locale.value === "zh"
    ? "zh-CN"
    : locale.value === "ja"
      ? "ja-JP"
      : locale.value === "fr"
        ? "fr-FR"
        : "en-US",
);
export const number = (value: number) =>
  new Intl.NumberFormat(localeDate.value, { maximumFractionDigits: 3 }).format(
    value,
  );
export function pickupBy(value: string) {
  return locale.value === "zh"
    ? `${value} 前领取`
    : `${t("beforePickup")} ${value}`;
}
const titles = {
  zh: "让善意有迹可循",
  en: "Good food, shared with care",
  fr: "Partager les aliments avec soin",
  ja: "思いやりを、記録に",
};
watchEffect(() => {
  document.documentElement.lang = localeDate.value;
  document.title = `RescueProof · ${titles[locale.value]}`;
  document
    .querySelector('meta[name="description"]')
    ?.setAttribute("content", t("heroText"));
});
export function notificationText(message: string) {
  if (message === workflow["notification.claimed"][1])
    return t("notification.claimed");
  if (message === workflow["notification.handedOver"][1])
    return t("notification.handedOver");
  return message;
}
