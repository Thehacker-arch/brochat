import { createRouter, createWebHistory } from 'vue-router'
import Login from '@/views/Login.vue'
import Chat from '@/views/Chat.vue'
import Register from '@/views/Register.vue'
import { isAuthenticated } from '@/api/auth'

const routes = [
  {
    path: '/',
    redirect: '/chat'
  },
  {
    path: '/login',
    name: 'Login',
    component: Login,
    meta: { requiresAuth: false }
  },
  {
    path: '/chat',
    name: 'Chat',
    component: Chat,
    meta: { requiresAuth: true }
  },
  {
    path: '/register',
    name:'Register',
    component: Register ,
    meta: { requiresAuth: false }
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

// Navigation guard
router.beforeEach((to, from, next) => {
  const isAuth = isAuthenticated()
  
  if (to.meta.requiresAuth && !isAuth) {
    // Redirect to login if trying to access protected route
    next('/login')
  } else if (to.path === '/login' && isAuth) {
    // Redirect away from login if already authenticated
    next('/chat')
  } else {
    next()
  }
})

export default router