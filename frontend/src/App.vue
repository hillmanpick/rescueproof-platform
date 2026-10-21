<script setup lang="ts">
import {onMounted,ref} from 'vue'
import {useRouter} from 'vue-router'
import {Leaf,ArrowUpRight,Menu,X,LogOut} from 'lucide-vue-next'
import {session,refreshSession,post} from './api'
const menu=ref(false),connectionError=ref('');const router=useRouter()
onMounted(()=>refreshSession().catch(()=>{connectionError.value='服务暂时不可用，部分功能需要后端连接。'}))
async function logout(){await post('/auth/logout');session.user=null;session.organizations=[];menu.value=false;router.push('/')}
</script>
<template>
  <div class="notice">让余量成为馈赠，让每一份善意有迹可循。<span>FOR FOOD. FOR COMMUNITY.</span></div>
  <header class="site-header"><div class="container header-inner">
    <RouterLink to="/" class="brand" aria-label="RescueProof 首页"><span class="brand-icon"><Leaf :size="23"/></span>Rescue<span class="brand-light">Proof</span><span class="brand-dot">®</span></RouterLink>
    <button class="mobile-menu icon-button" @click="menu=!menu" :aria-label="menu?'关闭菜单':'打开菜单'"><X v-if="menu"/><Menu v-else/></button>
    <nav :class="{expanded:menu}" @click="menu=false"><RouterLink to="/">发现余量</RouterLink><RouterLink to="/#how">如何参与</RouterLink><RouterLink to="/workspace">我的工作台</RouterLink><RouterLink v-if="session.user?.is_admin" to="/admin">组织管理</RouterLink></nav>
    <div class="header-actions"><template v-if="session.user"><RouterLink to="/workspace" class="account-name">{{session.user.name}}</RouterLink><button class="icon-button" aria-label="退出登录" @click="logout"><LogOut :size="18"/></button></template><RouterLink v-else to="/login" class="login-link">登录</RouterLink><RouterLink to="/share" class="button small">分享余量 <ArrowUpRight :size="16"/></RouterLink></div>
  </div></header>
  <div v-if="connectionError" class="connection-banner" role="status">{{connectionError}} <button @click="connectionError='';refreshSession().catch(()=>connectionError='服务暂时不可用，请稍后重试。')">重试连接</button></div>
  <main><RouterView :key="$route.path"/></main>
  <footer><div class="container footer-inner"><div><RouterLink class="brand" to="/"><Leaf :size="23"/> RescueProof</RouterLink><p>食物不应被浪费，善意值得被看见。</p></div><div class="footer-meta"><span>从一份余量，到一份温暖。</span><small>RescueProof · Community food rescue pilot</small></div></div><div class="container footer-bottom">© {{new Date().getFullYear()}} RescueProof <span>食品安全与现场交接由参与组织负责。记录证明用于核验数据完整性。</span></div></footer>
</template>
