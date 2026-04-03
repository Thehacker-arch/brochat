import axios, { AxiosError } from 'axios'
import { API_BASE_URL } from '../config'

export interface LoginResponse {
  token: string
  user: {
    id: string
    username: string
  }
}


export const register = async (credentials: {
  username: string
  password: string
}) => {
  try {
    const response = await axios.post(
      `${API_BASE_URL}/register`,
      credentials,
      {
        headers: {
          'Content-Type' : 'application/json',
        },
        withCredentials: true
      }
    )
    // if (!response.data?.token) {
    //     throw new Error('No token received')
    //   }
      return response.data
  } 
  catch (err) {
    const error = err as AxiosError<{ message: string }>
    console.error('Registration error:', error.response?.data)
    throw new Error(
      error.response?.data?.message || 
      error.message || 
      'Registration failed'
    )
  }
}

export const login = async (credentials: {
  username: string
  password: string
}): Promise<LoginResponse> => {
  try {
    const response = await axios.post<LoginResponse>(
      `${API_BASE_URL}/login`,
      credentials,
      {
        headers: {
          'Content-Type': 'application/json'
        },
        withCredentials: true
      }
    )
    
    if (!response.data?.token) {
      throw new Error('No token received')
    }
    
    localStorage.setItem('authToken', response.data.token)
    axios.defaults.headers.common['Authorization'] = `Bearer ${response.data.token}`
    
    return response.data
  } catch (err) {
    const error = err as AxiosError<{ message: string }>
    console.error('Login error:', error.response?.data)
    throw new Error(
      error.response?.data?.message || 
      error.message || 
      'Login failed'
    )
  }
}

export const isAuthenticated = (): boolean => {
  const token = localStorage.getItem('authToken'); // Or check sessionStorage/cookie
  return !!token;
}
  
  export const getAuthToken = (): string | null => {
    return localStorage.getItem('authToken')
  }
  
  export const clearAuth = (): void => {
    localStorage.removeItem('authToken')
  }


export const logout = (): void => {
    localStorage.removeItem('authToken')
    localStorage.removeItem('user')
    delete axios.defaults.headers.common['Authorization']
  }
  