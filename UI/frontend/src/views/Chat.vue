<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { logout, isAuthenticated } from '@/api/auth'
import axios from 'axios'

const router = useRouter()
const ws = ref<WebSocket | null>(null)

interface ChatMessage {
    username: string
    message: string
    time: string | Date
    timestamp: string
    isCurrentUser?: boolean
    avatar_url: string
    upload_url: string | null
}

interface User {
    id: string
    username: string
    avatar_url: string
}


const conversations = ref<Record<string, ChatMessage[]>>({
    public: []
})
const messages = ref<ChatMessage[]>([])
interface DateDivider {
    type: 'date-divider'
    date: Date
    displayText: string
}
type ChatItem = ChatMessage | DateDivider

function isDateDivider(item: ChatItem): item is DateDivider {
    return 'type' in item && item.type === 'date-divider'
}

const messagesWithDividers = computed<ChatItem[]>(() => {
    const result: ChatItem[] = []
    let lastDate: string | null = null

    const sortedMessages = [...messages.value].sort((a, b) =>
        new Date(a.time).getTime() - new Date(b.time).getTime()
    )

    for (const msg of sortedMessages) {
        const msgDate = new Date(msg.timestamp)
        const currentDateStr = msgDate.toDateString()

        if (lastDate === null || currentDateStr !== lastDate) {
            result.push({
                type: 'date-divider',
                date: msgDate,
                displayText: formatDividerDate(msgDate)
            })
            lastDate = currentDateStr
        }

        result.push(msg)
    }

    return result
})


function formatDividerDate(date: Date): string {
    const today = new Date()
    const yesterday = new Date(today)
    yesterday.setDate(yesterday.getDate() - 1)

    if (date.toDateString() === today.toDateString()) {
        return 'Today'
    }
    else if (date.toDateString() === yesterday.toDateString()) {
        return 'Yesterday'
    }
    else {
        return date.toLocaleDateString(undefined, {
            weekday: 'long',
            month: 'long',
            day: 'numeric',
            year: 'numeric'
        })
    }
}



const inputMessage = ref('')
const user = ref<User>({ id: 'unknown', username: 'unknown', avatar_url: 'unknown'})
const showDropdown = ref(false)
const showProfile = ref(false)
const sidebarVisible = ref(true)
const allUsers = ref<string[]>([])
const existingDMs = ref<string[]>([])
const currentChat = ref('public')
const fileInput = ref<HTMLInputElement | null>(null)
const selectedFile = ref<File | null>(null)
const chatContainer = ref<HTMLElement | null>(null)


function shouldGroupWithPrevious(index: number): boolean {
    if (index === 0) return false

    const currentItem = messagesWithDividers.value[index]
    const prevItem = messagesWithDividers.value[index - 1]

    // Don't group if previous item is a divider
    if (isDateDivider(prevItem)) return false

    // Don't group if different users
    // if (currentItem.username !== prevItem.username) return false

    // Don't group if time gap is more than 5 minutes
    const currentTime = new Date().getTime()
    const prevTime = new Date(prevItem.timestamp).getTime()
    return (currentTime - prevTime) < (5 * 60 * 1000)
}

function getUserAvatar(avatarUrl: string): string {
    const u = `${URL}${avatarUrl}`
    console.log(u)
    return `${URL}${avatarUrl}` 
}

function getImage(upload_url: string): string {
    const u = `${URL}${upload_url}`
    console.log(u)
    return `${URL}${upload_url}`
}

function formatMessageTime(timestamp: string): string {
    const date = new Date(timestamp)
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

const scrollToBottom = async () => {
    await nextTick()
    if (chatContainer.value) {
        chatContainer.value.scrollTop = chatContainer.value.scrollHeight
    }
}

const getConvoKey = (username1: string, username2: string) => {
    return [username1, username2].sort().join('-')
}
const URL = 'http://192.168.1.45:3000';
const URL2 = '192.168.1.45:3000';

const fetchAllUsers = async () => {
    try {
        const res = await fetch(`${URL}/users`)
        const data = await res.json()
        allUsers.value = data.map((u: any) => u.username)
    } catch (error) {
        console.error('Error fetching users:', error)
    }
}


const loadPublicMessages = async () => {
    try {
        const response = await axios.get(`${URL}/public`)

        conversations.value.public = response.data.map((msg: any) => ({
            username: msg.sender,
            message: msg.message,
            time: new Date(msg.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
            timestamp: new Date(msg.timestamp).toISOString(),
            isCurrentUser: msg.username === user.value.username,
            upload_url: msg.upload_url,
            avatar_url: msg.avatar_url
        }))

        if (currentChat.value === 'public') {
            messages.value = [...conversations.value.public]
            scrollToBottom()
        }
    } catch (err) {
        console.error('Failed to load public messages:', err)
    }
}


const loadDMMessages = async (targetUsername: string) => {
    try {
        const token = localStorage.getItem('authToken');
        if (!token) {
            console.error('No token found');
            return;
        }

        const convoKey = getConvoKey(user.value.username, targetUsername)
        const response = await axios.get(`${URL}/api/dm/${targetUsername}?current_user=${user.value.username}`,
            {
                headers: {
                    'Authorization': `Bearer ${token}`
                }
            })
        console.log("NIGGA UR RESPONSE: ", response.data);

        // Add messages to conversation history
        conversations.value[convoKey] = response.data.map((msg: any) => ({
            username: msg.sender === user.value.username ? user.value.username : msg.sender,
            message: msg.message,
            time: new Date(msg.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
            timestamp: new Date(msg.timestamp).toISOString(),
            isCurrentUser: msg.sender === user.value.username,
            upload_url: msg.upload_url,
            avatar_url: msg.avatar_url
        }))

        // Update current view if we're looking at this conversation
        if (currentChat.value === targetUsername || currentChat.value === 'public') {
            messages.value = [...conversations.value[convoKey]]
            console.log("CHAT: ", currentChat.value)
            scrollToBottom()
        }
    } catch (err) {
        console.error("Failed to load DM messages", err)
    }
}


const startNewDM = (targetUsername: string) => {
    if (targetUsername === user.value.username) return;

    const convoKey = getConvoKey(user.value.username, targetUsername)

    if (!existingDMs.value.includes(targetUsername)) {
        existingDMs.value = [...existingDMs.value, targetUsername]
        localStorage.setItem('dms', JSON.stringify(existingDMs.value))
    }

    if (!conversations.value[convoKey]) {
        conversations.value[convoKey] = []
        loadDMMessages(targetUsername)
    }

    currentChat.value = targetUsername
    messages.value = [...conversations.value[convoKey]]
    scrollToBottom()
    showDropdown.value = false
}

const switchToPublicChat = () => {
    currentChat.value = 'public'
    messages.value = [...conversations.value.public]
    scrollToBottom()
}


const parseJWT = (token: string): {
     id: string, username: string, avatarUrl: string
} => {
    try {
        const payload = token.split('.')[1]
        const decoded = JSON.parse(atob(payload))
        return {
            id: decoded.sub || 'unknown',
            username: decoded.username || 'unknown',
            avatarUrl: decoded.avatarUrl || 'unknown'
        }
    } catch (e) {
        console.error('Token parse failed', e)
        return { id: 'unknown', username: 'unknown', avatarUrl: 'unknown' }
    }
}

const handleAvatarUpload = async (event: Event) => {
    const input = event.target as HTMLInputElement
    if (!input.files?.length) return
    
    const file = input.files[0]
    const validTypes = ['image/jpeg', 'image/png', 'image/gif', 'image/webp']

    if (!validTypes.includes(file.type)) {
        throw new Error('Invalid file type. Please upload an image (JPEG, PNG, GIF, WEBP)')
    }
    if (file.size > 10 * 1024 * 1024) { alert('Avatar must be less than 10MB'); return }
    
    if (input.value) {
        const formData = new FormData()
        formData.append('avatar', file)
        formData.append('user_id', user.value.id)

        try {
            const token = localStorage.getItem('authToken') || ''
            const response = await axios.post(`${URL}/api/avatar-upload`, formData, {
                headers: {

                    'Authorization': `Bearer ${token}`,
                }
            })
            user.value.avatar_url = response.data.avatar_url

            if (currentChat.value === 'public') {
                await loadPublicMessages()
            } 
            else {
                await loadDMMessages(currentChat.value)
            }
        }
        catch (error) {
            console.error('Avatar upload failed:', error)
            alert('Failed to upload avatar. Please try again.')
        }
        finally {
            const newInput = input.cloneNode(true) as HTMLInputElement
            input.parentNode?.replaceChild(newInput, input)
            newInput.addEventListener('change', handleAvatarUpload)
        }
    }
}

const sendMessage = async () => {
    if (selectedFile.value) {
        const formData = new FormData()
        formData.append('file', selectedFile.value)
        formData.append('sender', user.value.username)
        formData.append('chat', currentChat.value)

        try {
            const token = localStorage.getItem('authToken') || ''
            const response = await axios.post(`${URL}/api/upload`, formData, {
                headers: {
                    'Content-Type': 'multipart/form-data',
                    'Authorization': `Bearer ${token}`,
                }
            })
            const uploadURL: string = response.data.upload_url
            const msg = {
                type: currentChat.value === 'public' ? 'chat' : 'dm',
                message: inputMessage.value,
                upload_url: uploadURL,
                ...(currentChat.value !== 'public' && { to: currentChat.value })
            }
            const newMsg = {
                username: user.value.username,
                message: `📎 Sent a file: ${selectedFile.value.name}`,
                time: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
                timestamp: new Date().toISOString(),
                isCurrentUser: true,
                upload_url: uploadURL,
                avatar_url: user.value.avatar_url
            }

            const convoKey = currentChat.value === 'public' ? 'public'
                    : getConvoKey(user.value.username, currentChat.value)

            if (!conversations.value[convoKey]) {
                conversations.value[convoKey] = []
            }

            conversations.value[convoKey].push(newMsg)
            messages.value.push(newMsg)
            selectedFile.value = null
            scrollToBottom()

            if (ws.value) {
                ws.value.send(JSON.stringify(msg))
            } else {
                console.warn('WebSocket is not connected.')
            }
        } catch (err) {
            console.error('File upload failed:', err)
        }
    }
    else if (inputMessage.value.trim() !== '') {
        sendTextMessage()
    }
}

const sendTextMessage = () => {
    if (!ws.value || !inputMessage.value.trim()) return

    const msg = {
        type: currentChat.value === 'public' ? 'chat' : 'dm',
        message: inputMessage.value,
        ...(currentChat.value !== 'public' && { to: currentChat.value })
    }
    // const uploadURL = null
    const newMsg = {
        username: user.value.username,
        message: inputMessage.value,
        type: 'text',
        time: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
        timestamp: new Date().toISOString(),
        isCurrentUser: true,
        upload_url: null,
        avatar_url: user.value.avatar_url
    }

    const convoKey = currentChat.value === 'public' ? 'public' :
        getConvoKey(user.value.username, currentChat.value)

    if (!conversations.value[convoKey]) {
        conversations.value[convoKey] = []
    }

    conversations.value[convoKey].push(newMsg)
    messages.value.push(newMsg)
    scrollToBottom()

    ws.value.send(JSON.stringify(msg))
    inputMessage.value = ''
}


const onFileSelected = () => {
    selectedFile.value = fileInput.value?.files?.[0] || null
}


onMounted(async () => {
    if (!isAuthenticated()) {
        router.push('/login')
        return
    }

    const savedDMs = localStorage.getItem('dms')
    if (savedDMs) existingDMs.value = JSON.parse(savedDMs)

    const rawToken = localStorage.getItem('authToken') || ''

    try {
        const res = await axios.get(`${URL}/api/me`, {
            headers: {
                Authorization: `Bearer ${rawToken}`
            }
        })
        console.log("RESPONSE API ME:  " + res.data.avatar_url)

        user.value = {
            ...res.data
        }
        console.log("avatar URL:  "+user.value.avatar_url)
    } catch (err) {
        console.error("Failed to load user:", err)
        router.push('/login')
    }

    await fetchAllUsers()

    for (const dm of existingDMs.value) {
        await loadDMMessages(dm)
    }
    await loadPublicMessages()

    messages.value = [...conversations.value.public]

    try {
        ws.value = new WebSocket(`ws://${URL2}/ws/${user.value.username}`)

        ws.value.onmessage = (event) => {
            const msgObj = JSON.parse(event.data)
            const now = new Date()
            const formattedTime = now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
            const timeraw = now.toISOString()

            if (msgObj.type === "dm") {
                const convoKey = getConvoKey(msgObj.from, user.value.username)

                if (!conversations.value[convoKey]) {
                    conversations.value[convoKey] = []
                }

                const newMsg = {
                    username: msgObj.from,
                    message: msgObj.message,
                    time: formattedTime,
                    timestamp: timeraw,
                    isCurrentUser: false,
                    upload_url: msgObj.upload_url,
                    avatar_url: msgObj.avatar_url
                }

                // Only add if not already present (prevent duplicates)
                if (!conversations.value[convoKey].some(m =>
                    m.username === newMsg.username &&
                    m.message === newMsg.message &&
                    Math.abs(new Date(m.time).getTime() - now.getTime()) < 1000
                )) {
                    conversations.value[convoKey].push(newMsg)

                    // Update view if we're looking at this conversation
                    if (currentChat.value === msgObj.from) {
                        messages.value = [...conversations.value[convoKey]]
                    }
                }
            }
            else if (msgObj.type === "chat") {
                // Only process if message is from someone else
                if (msgObj.username !== user.value.username) {
                    const newMsg = {
                        username: msgObj.username,
                        message: msgObj.message,
                        time: formattedTime,
                        timestamp: timeraw,
                        isCurrentUser: false,
                        avatar_url: msgObj.avatar_url,
                        upload_url: msgObj.upload_url
                    }

                    // Only add if not already present
                    if (!conversations.value.public.some(m =>
                        m.username === newMsg.username &&
                        m.message === newMsg.message &&
                        Math.abs(new Date(m.time).getTime() - now.getTime()) < 1000 // 1 second window
                    )) {
                        conversations.value.public.push(newMsg)

                        if (currentChat.value === 'public') {
                            messages.value = [...conversations.value.public]
                        }
                    }
                }
            }
        }

        ws.value.onopen = () => console.log('WebSocket connected')
        ws.value.onclose = () => console.log('WebSocket disconnected')
    } catch (err) {
        console.error('WebSocket connection failed:', err)
    }
})

onBeforeUnmount(() => {
    if (ws.value) {
        ws.value.close()
    }
})
const handleLogout = () => {
    logout()
    router.push('/login')
}

const toggleSidebar = () => {
    sidebarVisible.value = !sidebarVisible.value
}

</script>

<template>
    <div class="chat-wrapper">
        <div class="topbar">
            <button class="logout-button" @click="handleLogout">Logout</button>
        </div>
        <aside :class="['sidebar', { collapsed: !sidebarVisible }]">
            <button class="toggle-sidebar" @click="toggleSidebar">☰</button>

            <template v-if="sidebarVisible">
                <div class="dm-section">
                    <button class="sidebar-button" @click="switchToPublicChat">
                        Public Channel
                    </button>
                </div>

                <div class="dm-section">
                    <button class="sidebar-button" @click="showDropdown = !showDropdown">
                        Direct Messages ▾
                    </button>
                    <ul v-if="showDropdown" class="dropdown">
                        <!-- Scrollable container if more than 5 DMs -->
                        <div :class="['dropdown-scroll', { 'scroll-enabled': existingDMs.length > 5 }]">
                            <li v-for="dm in existingDMs" :key="dm" @click="startNewDM(dm)" class="dropdown-user">
                                📨 {{ dm }}
                            </li>
                        </div>

                        <!-- New DM options -->
                        <!-- <li v-for="user in allUsers.filter(u => !existingDMs.includes(u) && u !== user.username)"
                            :key="user" @click="startNewDM(user)" class="dropdown-user">
                            ➕ {{ user }}
                        </li> -->

                        <li v-for="filteredUser in allUsers.filter(u => !existingDMs.includes(u) && u !== user.username)"
                            :key="filteredUser" @click="startNewDM(filteredUser)" class="dropdown-user">
                            ➕ {{ filteredUser }}
                        </li>
                    </ul>
                </div>

                <button class="profile-button" @click="showProfile = !showProfile">
                    Profile
                </button>

                <div v-if="showProfile" class="profile-card">
                    <p><strong>Username:</strong> {{ user.username }}</p><br>
                    <p><strong>ID:</strong> {{ user.id }}</p><br>

                    <!-- Profile Picture with Upload -->
                    <div class="avatar-upload">
                        <label for="avatar-upload">
                            <img :src="getUserAvatar(user.avatar_url)" class="user-avatar" alt="Profile Picture">
                            <!-- <button @click="handleAvatarUpload">upload -->
                            <input id="avatar-upload" type="file" accept="image/*" @change="handleAvatarUpload" style="display: none;">
                            <!-- </button> -->
                        </label>
                    </div>
                </div>
            </template>
        </aside>

        <main class="chat-area">
            <h1 class="chat-title" data-text="broChat">broChat</h1>
            <h1 class="chat-usr">
                {{ currentChat === 'public' ? 'Public Chat' : `Direct Messaging ${currentChat}` }}
            </h1>


            <div class="chat-box discord-style" ref="chatContainer">
                <template v-for="(item, index) in messagesWithDividers" :key="index">
                    <div v-if="isDateDivider(item)" class="date-divider">
                        <span class="divider-line"></span>
                        <span class="divider-text">{{ item.displayText }}</span>
                        <span class="divider-line"></span>
                    </div>

                    <!-- Message Group -->
                    <div v-else class="message-group" :class="{ 'consecutive': index }">

                        <div v-if="index" class="avatar-container">
                            <img :src="getUserAvatar(item.avatar_url)" class="user-avatar" alt="Profile" />
                        </div>

                        <div class="message-content">
                            <div v-if="index" class="message-header">
                                <span class="msg-text">{{ item.username }}</span>
                                <span class="msg-time">{{ formatMessageTime(item.timestamp) }}</span>
                            </div>

                            <div class="message-bubble">
                                <template v-if="item.message">
                                    {{ item.message }}
                                </template>
                                <template v-if="item.upload_url">
                                    <img :src="getImage(item.upload_url)" class="uploaded-image" />
                                </template>
                            </div>
                        </div>
                    </div>
                </template>
            </div>


            <!-- <div class="chat-box" ref="chatContainer">
                <template v-for="(item, index) in messagesWithDividers" :key="index">
                    <div v-if="isDateDivider(item)" class="date-divider">
                        <span class="divider-line"></span>
                        <span class="divider-text">{{ item.displayText }}</span>
                        <span class="divider-line"></span>
                    </div>


                    <div v-else :class="['message-bubble', {
                        'sent': item.isCurrentUser,
                        'received': !item.isCurrentUser
                    }]">
                        <span class="msg-text"><strong>{{ item.username }}:</strong>
                            {{ item.message }}
                        </span>
                        <span class="msg-time">{{ item.time }}</span>
                    </div>
                </template>
            </div> -->


            <!-- 
            <div class="chat-box" ref="chatContainer">
                <div v-for="(msg, index) in messages" :key="index"
                    :class="['message-bubble', { 'sent': msg.isCurrentUser, 'received': !msg.isCurrentUser }]">

                    <div v-if="'dateMarker' in msg" class="date-marker">
                        {{ formatDateMarker(msg.time) }}
                    </div>
    

                    <span class="msg-text"><strong>{{ msg.username }}:</strong>
                        {{ msg.message }}

                    </span>
                    <span class="msg-time">{{ msg.time }}</span>
                </div>
            </div> -->

            <div class="chat-input">
                <input type="file" ref="fileInput" style="display: none;" @change="onFileSelected" />
                <button @click="fileInput?.click()" class="clip-button">📎</button>

                <input v-model="inputMessage" type="text" placeholder="Type a message..." class="input"
                    @keyup.enter="sendMessage" />
                <button class="send-button" @click="sendMessage">Send</button>
            </div>
        </main>
    </div>
</template>

<style scoped>
.uploaded-image {
  max-width: 400px;
  max-height: 300px;
  width: auto;
  height: auto;
  border-radius: 4px;
  background-color: #f0f0f0;
  display: block;
}

.discord-style {
    padding: 16px;
    background-color: #36393f;
    color: #dcddde;
    font-family: 'Whitney', 'Helvetica Neue', Helvetica, Arial, sans-serif;
}

.message-group {
    display: flex;
    padding: 4px 0;
    margin-left: 16px;
    position: relative;
}

.message-group.consecutive {
    margin-top: -2px;
    padding-top: 2px;
}

.avatar-container {
    margin-right: 16px;
    flex-shrink: 0;
}

.user-avatar {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    object-fit: cover;
}


.date-divider {
    display: flex;
    align-items: center;
    margin: 1.5rem 0;
    color: #666;
    font-size: 0.8rem;
    text-align: center;
}

.divider-line {
    flex: 1;
    height: 1px;
    background-color: rgba(240, 231, 231, 0.1);
    margin: 0 0.8rem;
}

.divider-text {
    padding: 0 0.5rem;
    color: #666;
    font-weight: 500;
}


.clip-button {
    background: transparent;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    margin-right: 0.5rem;
}

.message-bubble {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 0.5rem;
    margin-bottom: 0.5rem;
    /* background-color: #323030; */
    border-radius: 6px;
    font-size: 0.95rem;
    max-width: 100%;
    word-break: break-word;
}

.chat-usr {
    font-family: Verdana;
    font-size: 16px;
    font-weight: 100;
    padding-bottom: 6px;
    padding-top: 10px;
}

.msg-text {
    flex-grow: 1;
    overflow-wrap: anywhere;
    min-width: 0;
}

.msg-time {
    margin-left: 1rem;
    font-size: 0.8rem;
    color: #aaa;
    white-space: nowrap;
    /* Add: */
    flex-shrink: 0;
    /* Prevents time from being squeezed */
}

.chat-wrapper {
    display: flex;
    height: 100vh;
    background-color: #101011;
    color: #fff;
    font-family: 'Segoe UI', sans-serif;
    overflow: hidden;
    /* Add: */
    min-width: 0;
    /* Important for flex children */
}


.topbar {
    position: absolute;
    top: 0;
    right: 0;
    padding-right: 16px;
    padding-bottom: 8px;
    z-index: 10;
    display: flex;
    gap: 0.5rem;
    align-items: center;
}


.toggle-sidebar:hover {
    background-color: #444;
}

.sidebar {
    width: 250px;
    min-width: 50px;
    transition: width 0.3s ease;
    overflow: hidden;
    padding: 1rem;
    background-color: #1e1e1e;
    display: flex;
    flex-direction: column;
    position: relative;
    flex-shrink: 0;
}

.sidebar.collapsed {
    width: 10px;
    padding: 1rem 0.5rem;
}

.toggle-sidebar {
    background-color: transparent;
    color: white;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    margin-bottom: 1rem;
}


.sidebar-button {
    background: #292929;
    color: white;
    border: none;
    padding: 0.75rem;
    margin-bottom: 1rem;
    cursor: pointer;
    border-radius: 6px;
    text-align: left;
}

.dropdown {
    background: #2e2e2e;
    list-style: none;
    padding-left: 0;
    margin-top: 0.5rem;
    border-radius: 4px;
    overflow: hidden;
    max-height: 400px;
    /* Adjust as needed */
    overflow-y: auto;
}

.dropdown-scroll {
    max-height: 200px;
    /* Height for scrollable area */
    overflow-y: auto;
    border-top: 1px solid #444;
    border-bottom: 1px solid #444;
    margin: 0.25rem 0;
}

.dropdown-user {
    padding: 0.5rem;
    border-bottom: 1px solid #444;
    cursor: pointer;
}

.dropdown-user:hover {
    background-color: #3a3a3a;
}

/* Make scrollbar prettier */
.dropdown-scroll::-webkit-scrollbar {
    width: 6px;
}

.dropdown-scroll::-webkit-scrollbar-track {
    background: #2e2e2e;
}

.dropdown-scroll::-webkit-scrollbar-thumb {
    background: #555;
    border-radius: 3px;
}

.dropdown-scroll::-webkit-scrollbar-thumb:hover {
    background: #777;
}


.profile-button {
    margin-top: auto;
    background-color: #333;
    border: none;
    color: white;
    padding: 0.5rem;
    cursor: pointer;
    border-radius: 6px;
}

.profile-card {
    background-color: #2a2a2a;
    padding: 1rem;
    margin-top: 0.5rem;
    border-radius: 6px;
    font-size: 0.9rem;
}

.chat-area {
    flex-grow: 1;
    display: flex;
    flex-direction: column;
    padding: 1rem;
    min-width: 0;
    overflow: hidden;
}

.chat-title {
    font-size: 2.6rem;
    font-family: 'Poppins', sans-serif;
    /* Create the gradient on a pseudo-element instead */
    position: relative;
    display: inline-block;
    color: transparent;
    /* Hide the text to show the gradient */
}

.chat-title::before {
    content: attr(data-text);
    position: absolute;
    background: linear-gradient(100deg, #000000, #63626b, #27d704);
    -webkit-background-clip: text;
    background-clip: text;
    /* Ensure full gradient is always visible */
    background-size: 200% 200%;
    background-position: center;
    /* Other text styling */
    text-transform: capitalize;
    font-weight: 400;
    letter-spacing: 1.5px;
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    /* Match the parent element's text */
    font: inherit;
}

.chat-box {
    flex-grow: 1;
    /* background-color: #121212; */
    background-color: #101011;
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1rem;
    overflow-y: auto;
    overflow-x: hidden;
    word-break: break-word;
    max-width: 100%;
}

.chat-input {
    display: flex;
    gap: 0.5rem;
    align-items: center;
}

.input {
    flex-grow: 1;
    padding: 0.75rem;
    background: #2c2c2c;
    border: 1px solid #444;
    border-radius: 6px;
    color: white;
}


.send-button {
    padding: 0.80rem 2rem;
    background-color: #f56969;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    margin-bottom: auto;
}

.logout-button {
    margin-top: 1rem;
    background-color: #d32f2f;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    align-self: flex-start;
    cursor: pointer;
}
</style>