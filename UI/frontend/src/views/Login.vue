<template>
  <div class="login-container" @scroll.passive="onScroll" ref="scrollContainer">
    <div class="big-brochat" :style="brochatGradientStyle">BroChat</div>

    <div class="spacer"></div>

    <div class="login-box">
      <h1 class="title">Login</h1>
      <form @submit.prevent="handleLogin" class="login-form">
        <input v-model="username" placeholder="Username" class="input-field" />
        <input v-model="password" type="password" placeholder="Password" class="input-field" />
        <button type="submit" class="login-button">Login</button>
        <p v-if="error" class="error">{{ error }}</p>
      </form>
      <p class="redirect">
        Don't have an account?
        <router-link to="/register" style="color: #c3e0ff;">Register</router-link>
      </p>
    </div>

    <div class="bottom-space"></div>
  </div>
</template>

<script setup lang="ts">
  import { ref, onMounted, computed } from 'vue'
  import { useRouter } from 'vue-router'
  import { login, isAuthenticated } from '@/api/auth'

  const router = useRouter()
  const username = ref('')
  const password = ref('')
  const error = ref('')

  const scrollY = ref(0)
  const scrollContainer = ref<HTMLElement | null>(null)

  const onScroll = () => {
    if (scrollContainer.value) {
      scrollY.value = scrollContainer.value.scrollTop
    }
  }

  const handleLogin = async () => {
    try {
      error.value = ''
      await login({ username: username.value, password: password.value })
      router.push('/chat')
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Login failed'
      console.error('Login error:', err)
    }
  }

  const brochatGradientStyle = computed(() => {
      const progress = Math.min(scrollY.value / 250, 1)
      const blue = `rgba(0, 255, 255, ${1 - progress})`
      const yellow = `rgba(255, 225, 0, ${progress})`
      return {
          background: `linear-gradient(to right, ${blue}, ${yellow})`,
          WebkitBackgroundClip: 'text',
          WebkitTextFillColor: 'transparent'
      }
    })

  onMounted(() => {
    if (isAuthenticated()) {
      router.push('/chat')
    }
})
</script>

<style scoped>
.login-container {
  height: 100vh;
  overflow-y: auto;
  background: linear-gradient(135deg, #0a0f1a, #101d2d, #182b3a);
  animation: backgroundShift 10s ease infinite;
  font-family: 'Segoe UI', sans-serif;
  display: flex;
  flex-direction: column;
  align-items: center;
  scroll-behavior: smooth;
  position: relative;
  padding: 0 16px;
  box-sizing: border-box;
}

@keyframes backgroundShift {
  0% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
  100% { background-position: 0% 50%; }
}

.big-brochat {
  font-size: 5rem; /* Reduced from 15rem */
  font-weight: 900;
  text-align: center;
  margin-top: 20vh; /* Reduced from 20vh */
  margin-bottom: 5vh; /* Reduced from 10vh */
  transition: background 1s ease;
  user-select: none;
  pointer-events: none;
  text-shadow: 0 0 20px rgba(0, 198, 255, 0.3); /* Reduced glow */
  letter-spacing: 5px; /* Reduced from 20px */
  line-height: 1;
}

@media (min-width: 768px) {
  .big-brochat {
    font-size: 15rem;
    margin-top: 20vh;
    margin-bottom: 10vh;
    text-shadow: 0 0 40px rgba(0, 198, 255, 0.3);
    letter-spacing: 20px;
  }
}

.spacer {
  height: 0vh;
}

.login-box {
  position: relative;
  z-index: 10;
  background: rgba(0, 0, 0, 0.2);
  padding: 2.5rem; /* Slightly reduced */
  border-radius: 20px; /* Slightly reduced */
  width: 90%; /* Changed from 90% */
  max-width: 400px;
  margin-bottom: 20vh; /* Reduced from 20vh */
  backdrop-filter: blur(25px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 0 30px rgba(0, 153, 255, 0.05);
  box-sizing: border-box;
}

.title {
  text-align: center;
  color: #c3e0ff;
  font-size: 1.8rem; /* Slightly reduced */
  margin-bottom: 1.2rem; /* Slightly reduced */
  text-shadow: 0 0 8px rgba(144, 202, 249, 0.6);
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.input-field {
  padding: 0.75rem 1rem;
  border-radius: 10px;
  border: none;
  background-color: rgba(255, 255, 255, 0.05);
  color: white;
  font-size: 1rem;
  transition: 0.2s;
  -webkit-appearance: none; /* Removes iOS shadow */
}

.input-field:focus {
  outline: none;
  background-color: rgba(255, 255, 255, 0.1);
  box-shadow: 0 0 5px #00c6ff;
}

.login-button {
  padding: 0.75rem;
  background-color: #00c6ff;
  color: #000;
  font-weight: bold;
  border: none;
  border-radius: 10px;
  cursor: pointer;
  transition: 0.2s;
  font-size: 1rem;
  -webkit-tap-highlight-color: transparent; /* Removes tap highlight on mobile */
}

.login-button:hover {
  background-color: #00a9dd;
}

.error {
  color: #ff5c5c;
  text-align: center;
  margin-top: 0.5rem;
  font-size: 0.9rem; /* Slightly reduced */
}

.redirect {
  font-family: monospace;
  font-size: 0.9rem; /* Slightly reduced */
  color: #8aa9c5;
  text-align: center;
  margin-top: 1rem; /* Slightly reduced */
}

.bottom-space {
  height: 50vh; /* Reduced from 200vh */
}

/* Mobile-specific optimizations */
@media (max-width: 480px) {
  .login-box {
    padding: 1.5rem;
    border-radius: 12px;
  margin-bottom: 5vh;
  }
  
  .input-field {
    font-size: 0.95rem;
    padding: 0.7rem 0.9rem;
  }
  
  .login-button {
    padding: 0.7rem;
  }
}
</style>