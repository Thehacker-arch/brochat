import { createApp } from 'vue'
// In main.ts
import App from '@/App.vue'  // Note the .vue extension
import router from '@/router/index'  // .ts extension optional/ Make sure this path is correct
import '@/assets/main.css'


const app = createApp(App)
app.use(router)
app.mount('#app')